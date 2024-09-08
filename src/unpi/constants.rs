pub const BEACON_MAX_DEPTH: u8 = 0x0f;
pub const DEF_NWK_RADIUS: u8 = 2 * BEACON_MAX_DEPTH;

pub mod af {
    pub enum InterpanCtl {
        CTL = 0,
        SET = 1,
        REG = 2,
        CHK = 3,
    }

    pub enum NetworkLatencyReq {
        NoLatencyReqs = 0,
        FastBeacons = 1,
        SlowBeacons = 2,
    }

    pub enum Otions {
        Preprocess = 4,
        LimitConcentrator = 8,
        AckRequest = 16,
        DiscvRoute = 32,
        EnSecurity = 64,
        SkipRouting = 128,
    }

    pub const DEFAULT_RADIUS: u8 = super::DEF_NWK_RADIUS;
}
