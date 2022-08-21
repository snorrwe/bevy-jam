use bevy::{ecs::schedule::ShouldRun, prelude::*, transform::TransformSystem};

pub type LayerMask = u32;

#[derive(PartialEq)]
#[repr(u32)]
pub enum CollisionType {
    // use powers of two, at most 31 items
    // Expand the repr if you want more..
    Terrain = 1,
    Enemy = 1 << 1,
    Player = 1 << 2,
}

impl CollisionType {
    pub const NONE: LayerMask = 0;
    pub const TERRAIN: LayerMask = Self::Terrain as LayerMask;
    pub const ENEMY: LayerMask = Self::Enemy as LayerMask;
    pub const PLAYER: LayerMask = Self::Player as LayerMask;

    // special masks
    pub const PLAYER_COLLISIONS: LayerMask =
        CollisionType::ENEMY | CollisionType::TERRAIN;
    pub const ENEMY_COLLISIONS: LayerMask =
        CollisionType::PLAYER | CollisionType::TERRAIN;
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct CollisionFilter {
    pub self_layers: LayerMask,
    pub collisions_mask: LayerMask,
}

impl CollisionFilter {
    #[inline]
    pub fn collides_ty(self, ty: CollisionType) -> bool {
        (self.collisions_mask & (ty as LayerMask)) != 0
    }

    /// checks self layers against other collision
    ///
    /// not commutative!
    #[inline]
    pub fn collides(self, other: CollisionFilter) -> bool {
        (self.self_layers & other.collisions_mask) != 0
    }
}

struct AABBArray {
    aabbs: Vec<(Entity, AABB, CollisionFilter)>,
    sort_axis: usize,
}

#[derive(Clone, Debug, Copy)]
pub(crate) struct AABBCollision {
    pub entity1: Entity,
    pub entity2: Entity,
}

#[derive(Default, Clone, Debug, Component)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Default, Clone, Debug, Component, Reflect)]
pub struct AABBDescriptor {
    pub radius: Vec3,
}

#[derive(Default, Clone, Debug, Bundle)]
pub struct AABBBundle {
    pub desc: AABBDescriptor,
    pub filter: CollisionFilter,
    pub aabb: AABB,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

fn aabb_sort_sweep_system(
    mut e: EventWriter<AABBCollision>,
    mut collection: ResMut<AABBArray>,
    mut tick: Local<u64>,
    mut collisions_to_send: Local<Vec<AABBCollision>>,
    q: Query<(Entity, &AABB, &CollisionFilter)>,
) {
    let _e = tracing::debug_span!("AABB sort_sweep", tick = *tick).entered();
    *tick += 1;

    // rebuild the array
    collection.aabbs.clear();
    collection.aabbs.reserve(q.iter().len());
    for (e, a, f) in q.iter() {
        collection.aabbs.push((e, a.clone(), *f));
    }

    if collection.aabbs.len() < 2 {
        return;
    }

    // sort the array by the current axis
    let sort_axis = collection.sort_axis;
    collection
        .aabbs
        .sort_by(move |(_, aabb1, _), (_, aabb2, _)| {
            aabb1.min[sort_axis]
                .partial_cmp(&aabb2.min[sort_axis])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    // sweep for collisions
    let mut s = Vec3::ZERO;
    let mut s2 = Vec3::ZERO;
    collisions_to_send.clear();
    for (i, (e1, aabb1, f1)) in collection.aabbs.iter().enumerate() {
        let p = (aabb1.max + aabb1.min) * 0.5;
        // update sums
        s += p;
        s2 += p * p;
        // test collisions against all posible overlapping AABBs, following current one
        for (e2, aabb2, f2) in collection.aabbs[i + 1..].iter() {
            // stop when tested AABBs are beyond the end of the current AABB
            if aabb2.min[sort_axis] > aabb1.max[sort_axis] {
                break;
            }

            // `collides` is not commutative
            if (f1.collides(*f2) || f2.collides(*f1)) && aabb_aabb(aabb1, aabb2)
            {
                trace!(
                    "Collision between {:?} {:?} {:?} {:?}",
                    e1,
                    e2,
                    aabb1,
                    aabb2
                );
                collisions_to_send.push(AABBCollision {
                    entity1: *e1,
                    entity2: *e2,
                });
            }
        }
    }

    e.send_batch(collisions_to_send.drain(..));

    let variance = (s2 - s * s) / (collection.aabbs.len() as f32);

    // update sorting axis to be the one with the greatest variance
    collection.sort_axis = 0;
    if variance.y > variance.x {
        collection.sort_axis = 1;
    }
}

fn should_run(state: Res<State<crate::SceneState>>) -> ShouldRun {
    match state.current() {
        crate::SceneState::InGame => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

pub struct CollisionPlugin;

/// Collistion stages
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum Labels {
    UpdateBoxes,
    Sweep,
}

pub fn aabb_aabb(a: &AABB, b: &AABB) -> bool {
    for i in 0..3 {
        if a.max[i] < b.min[i] || a.min[i] > b.max[i] {
            return false;
        }
    }
    true
}

fn update_boxes(mut q: Query<(&mut AABB, &AABBDescriptor, &GlobalTransform)>) {
    q.for_each_mut(|(mut aabb, desc, tr)| {
        let radius = desc.radius * tr.compute_transform().scale;
        let radius = radius.abs();
        let pos = tr.translation();

        let min = pos - radius;
        let max = pos + radius;

        aabb.min = min;
        aabb.max = max;
    });
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AABBArray {
            aabbs: Vec::with_capacity(4096),
            sort_axis: 0,
        })
        .add_event::<AABBCollision>()
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(should_run)
                .with_system(update_boxes)
                .after(TransformSystem::TransformPropagate)
                .label(Labels::UpdateBoxes),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(should_run)
                .with_system(aabb_sort_sweep_system)
                .after(Labels::UpdateBoxes)
                .label(Labels::Sweep),
        );
    }
}
