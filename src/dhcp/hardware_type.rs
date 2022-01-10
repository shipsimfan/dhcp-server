pub enum HardwareType {
    Ethernet,
    ExperimentalEthernet,
    AmateurRadioAX25,
    ProteonProNetTokenRing,
    Chaos,
    IEEE802Networks,
    ARCNET,
    Hyperchannel,
    Lanstar,
    AutonetShortAddress,
    LocalTalk,
    LocalNet,
    UltraLink,
    SMDS,
    FrameRelay,
    AsynchronousTransmissionMode,
    HDLC,
    FibreChannel,
    SerialLine,
    Other(u8),
}

impl HardwareType {
    pub fn parse(value: u8) -> Self {
        match value {
            1 => HardwareType::Ethernet,
            2 => HardwareType::ExperimentalEthernet,
            3 => HardwareType::AmateurRadioAX25,
            4 => HardwareType::ProteonProNetTokenRing,
            5 => HardwareType::Chaos,
            6 => HardwareType::IEEE802Networks,
            7 => HardwareType::ARCNET,
            8 => HardwareType::Hyperchannel,
            9 => HardwareType::Lanstar,
            10 => HardwareType::AutonetShortAddress,
            11 => HardwareType::LocalTalk,
            12 => HardwareType::LocalNet,
            13 => HardwareType::UltraLink,
            14 => HardwareType::SMDS,
            15 => HardwareType::FrameRelay,
            16 | 19 | 21 => HardwareType::AsynchronousTransmissionMode,
            17 => HardwareType::HDLC,
            18 => HardwareType::FibreChannel,
            20 => HardwareType::SerialLine,
            _ => HardwareType::Other(value),
        }
    }

    pub fn generate(&self) -> u8 {
        match self {
            HardwareType::Ethernet => 1,
            HardwareType::ExperimentalEthernet => 2,
            HardwareType::AmateurRadioAX25 => 3,
            HardwareType::ProteonProNetTokenRing => 4,
            HardwareType::Chaos => 5,
            HardwareType::IEEE802Networks => 6,
            HardwareType::ARCNET => 7,
            HardwareType::Hyperchannel => 8,
            HardwareType::Lanstar => 9,
            HardwareType::AutonetShortAddress => 10,
            HardwareType::LocalTalk => 11,
            HardwareType::LocalNet => 12,
            HardwareType::UltraLink => 13,
            HardwareType::SMDS => 14,
            HardwareType::FrameRelay => 15,
            HardwareType::AsynchronousTransmissionMode => 16,
            HardwareType::HDLC => 17,
            HardwareType::FibreChannel => 18,
            HardwareType::SerialLine => 20,
            HardwareType::Other(value) => *value,
        }
    }
}

impl std::fmt::Display for HardwareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HardwareType::Ethernet => format!("Ethernet (10Mb)"),
                HardwareType::ExperimentalEthernet => format!("Experimental Ethernet (3Mb)"),
                HardwareType::AmateurRadioAX25 => format!("Amateur Radio AX.25"),
                HardwareType::ProteonProNetTokenRing => format!("Proteon ProNET Token Ring"),
                HardwareType::Chaos => format!("Chaos"),
                HardwareType::IEEE802Networks => format!("IEEE 802 Networks"),
                HardwareType::ARCNET => format!("ARCNET"),
                HardwareType::Hyperchannel => format!("Hyperchannel"),
                HardwareType::Lanstar => format!("Lanstar"),
                HardwareType::AutonetShortAddress => format!("Autonet Short Address"),
                HardwareType::LocalTalk => format!("LocalTalk"),
                HardwareType::LocalNet => format!("LocalNet (IBM PCNet or SYTEK LocalNET)"),
                HardwareType::UltraLink => format!("Ultra link"),
                HardwareType::SMDS => format!("SMDS"),
                HardwareType::FrameRelay => format!("Frame Relay"),
                HardwareType::AsynchronousTransmissionMode =>
                    format!("Asynchronous Transmission Mode (ATM)"),
                HardwareType::HDLC => format!("HDLC"),
                HardwareType::FibreChannel => format!("Fibre Channel"),
                HardwareType::SerialLine => format!("Serial Line"),
                HardwareType::Other(value) => format!("Other ({})", value),
            }
        )
    }
}
