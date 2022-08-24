use bevy::prelude::Component;

#[derive(Clone, Component)]
pub enum Easing {
    None,
    Linear,
    QuartOutInverted,
    QuartOut,
    OutElastic,
    PulsateInOutCubic,
    PulsateInOutCubicShifted,
}

impl Easing {
    pub fn get_easing(&self, percent: f32) -> f32 {
        match self {
            Easing::None => 1.,
            Easing::Linear => percent,
            Easing::QuartOutInverted => 1. - ezing::quart_out(percent),
            Easing::QuartOut => ezing::quart_out(percent),
            Easing::OutElastic => ezing::elastic_out(percent),
            Easing::PulsateInOutCubic => {
                if percent < 0.5 {
                    ezing::cubic_inout(percent * 2.)
                } else {
                    1. - ezing::circ_in((percent - 0.5) * 2.)
                }
            }
            Easing::PulsateInOutCubicShifted => {
                return Easing::PulsateInOutCubic.get_easing(percent) + 0.1;
            }
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::None
    }
}
