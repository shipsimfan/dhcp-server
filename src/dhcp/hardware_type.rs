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
}

impl HardwareType {
    pub fn parse(value: u8) -> Option<Self> {
        Some(match value {
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
            _ => return None,
        })
    }
}

impl std::fmt::Display for HardwareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HardwareType::Ethernet => "Ethernet (10Mb)",
                HardwareType::ExperimentalEthernet => "Experimental Ethernet (3Mb)",
                HardwareType::AmateurRadioAX25 => "Amateur Radio AX.25",
                HardwareType::ProteonProNetTokenRing => "Proteon ProNET Token Ring",
                HardwareType::Chaos => "Chaos",
                HardwareType::IEEE802Networks => "IEEE 802 Networks",
                HardwareType::ARCNET => "ARCNET",
                HardwareType::Hyperchannel => "Hyperchannel",
                HardwareType::Lanstar => "Lanstar",
                HardwareType::AutonetShortAddress => "Autonet Short Address",
                HardwareType::LocalTalk => "LocalTalk",
                HardwareType::LocalNet => "LocalNet (IBM PCNet or SYTEK LocalNET)",
                HardwareType::UltraLink => "Ultra link",
                HardwareType::SMDS => "SMDS",
                HardwareType::FrameRelay => "Frame Relay",
                HardwareType::AsynchronousTransmissionMode =>
                    "Asynchronous Transmission Mode (ATM)",
                HardwareType::HDLC => "HDLC",
                HardwareType::FibreChannel => "Fibre Channel",
                HardwareType::SerialLine => "Serial Line",
            }
        )
    }
}
