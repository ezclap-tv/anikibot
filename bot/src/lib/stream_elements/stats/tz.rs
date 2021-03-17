/// StreamElements requires int32 timezone indices which Chrono doesn't provide.
///
/// Adapted from [`https://support.microsoft.com/en-ca/help/973627/microsoft-time-zone-index-values`].
#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeZone {
    /// (GMT-12:00) International Date Line West
    DatelineStandardTime = 0x0,
    /// (GMT-11:00) Midway Island, Samoa
    SamoaStandardTime = 0x1,
    /// (GMT-10:00) Hawaii
    HawaiianStandardTime = 0x2,
    /// (GMT-09:00) Alaska
    AlaskanStandardTime = 0x3,
    /// (GMT-08:00) Pacific Time (US and Canada); Tijuana
    PacificStandardTime = 0x4,
    /// (GMT-07:00) Mountain Time (US and Canada)
    MountainStandardTime = 0xA,
    /// (GMT-07:00) Chihuahua, La Paz, Mazatlan
    MexicoStandardTime2 = 0xD,
    /// (GMT-07:00) Arizona
    USMountainStandardTime = 0xF,
    /// (GMT-06:00) Central Time (US and Canada)
    CentralStandardTime = 0x14,
    /// (GMT-06:00) Saskatchewan
    CanadaCentralStandardTime = 0x19,
    /// (GMT-06:00) Guadalajara, Mexico City, Monterrey
    MexicoStandardTime = 0x1E,
    /// (GMT-06:00) Central America
    CentralAmericaStandardTime = 0x21,
    /// (GMT-05:00) Eastern Time (US and Canada)
    EasternStandardTime = 0x23,
    /// (GMT-05:00) Indiana (East)
    USEasternStandardTime = 0x28,
    /// (GMT-05:00) Bogota, Lima, Quito
    SAPacificStandardTime = 0x2D,
    /// (GMT-04:00) Atlantic Time (Canada)
    AtlanticStandardTime = 0x32,
    /// (GMT-04:00) Georgetown, La Paz, San Juan
    SAWesternStandardTime = 0x37,
    /// (GMT-04:00) Santiago
    PacificSAStandardTime = 0x38,
    /// (GMT-03:30) Newfoundland
    NewfoundlandAndLabradorStandardTime = 0x3C,
    /// (GMT-03:00) Brasilia
    ESouthAmericaStandardTime = 0x41,
    /// (GMT-03:00) Georgetown
    SAEasternStandardTime = 0x46,
    /// (GMT-03:00) Greenland
    GreenlandStandardTime = 0x49,
    /// (GMT-02:00) Mid-Atlantic
    MidAtlanticStandardTime = 0x4B,
    /// (GMT-01:00) Azores
    AzoresStandardTime = 0x50,
    /// (GMT-01:00) Cape Verde Islands
    CapeVerdeStandardTime = 0x53,
    /// (GMT) Greenwich Mean Time: Dublin, Edinburgh, Lisbon, London
    GMTStandardTime = 0x55,
    /// (GMT) Monrovia, Reykjavik
    GreenwichStandardTime = 0x5A,
    /// (GMT+01:00) Belgrade, Bratislava, Budapest, Ljubljana, Prague
    CentralEuropeStandardTime = 0x5F,
    /// (GMT+01:00) Sarajevo, Skopje, Warsaw, Zagreb
    CentralEuropeanStandardTime = 0x64,
    /// (GMT+01:00) Brussels, Copenhagen, Madrid, Paris
    RomanceStandardTime = 0x69,
    /// (GMT+01:00) Amsterdam, Berlin, Bern, Rome, Stockholm, Vienna
    WEuropeStandardTime = 0x6E,
    /// (GMT+01:00) West Central Africa
    WCentralAfricaStandardTime = 0x71,
    /// (GMT+02:00) Minsk
    EEuropeStandardTime = 0x73,
    /// (GMT+02:00) Cairo
    EgyptStandardTime = 0x78,
    /// (GMT+02:00) Helsinki, Kiev, Riga, Sofia, Tallinn, Vilnius
    FLEStandardTime = 0x7D,
    /// (GMT+02:00) Athens, Bucharest, Istanbul
    GTBStandardTime = 0x82,
    /// (GMT+02:00) Jerusalem
    IsraelStandardTime = 0x87,
    /// (GMT+02:00) Harare, Pretoria
    SouthAfricaStandardTime = 0x8C,
    /// (GMT+03:00) Moscow, St. Petersburg, Volgograd
    RussianStandardTime = 0x91,
    /// (GMT+03:00) Kuwait, Riyadh
    ArabStandardTime = 0x96,
    /// (GMT+03:00) Nairobi
    EAfricaStandardTime = 0x9B,
    /// (GMT+03:00) Baghdad
    ArabicStandardTime = 0x9E,
    /// (GMT+03:30) Tehran
    IranStandardTime = 0xA0,
    /// (GMT+04:00) Abu Dhabi, Muscat
    ArabianStandardTime = 0xA5,
    /// (GMT+04:00) Baku, Tbilisi, Yerevan
    CaucasusStandardTime = 0xAA,
    /// (GMT+04:30) Kabul
    TransitionalIslamicStateofAfghanistanStandardTime = 0xAF,
    /// (GMT+05:00) Ekaterinburg
    EkaterinburgStandardTime = 0xB4,
    /// (GMT+05:00) Tashkent
    WestAsiaStandardTime = 0xB9,
    /// (GMT+05:30) Chennai, Kolkata, Mumbai, New Delhi
    IndiaStandardTime = 0xBE,
    /// (GMT+05:45) Kathmandu
    NepalStandardTime = 0xC1,
    /// (GMT+06:00) Astana, Dhaka
    CentralAsiaStandardTime = 0xC3,
    /// (GMT+06:00) Sri Jayawardenepura
    SriLankaStandardTime = 0xC8,
    /// (GMT+06:00) Almaty, Novosibirsk
    NCentralAsiaStandardTime = 0xC9,
    /// (GMT+06:30) Yangon (Rangoon)
    MyanmarStandardTime = 0xCB,
    /// (GMT+07:00) Bangkok, Hanoi, Jakarta
    SEAsiaStandardTime = 0xCD,
    /// (GMT+07:00) Krasnoyarsk
    NorthAsiaStandardTime = 0xCF,
    /// (GMT+08:00) Beijing, Chongqing, Hong Kong, Urumqi
    ChinaStandardTime = 0xD2,
    /// (GMT+08:00) Kuala Lumpur, Singapore
    SingaporeStandardTime = 0xD7,
    /// (GMT+08:00) Taipei
    TaipeiStandardTime = 0xDC,
    /// (GMT+08:00) Perth
    WAustraliaStandardTime = 0xE1,
    /// (GMT+08:00) Irkutsk, Ulaanbaatar
    NorthAsiaEastStandardTime = 0xE3,
    /// (GMT+09:00) Seoul
    KoreaStandardTime = 0xE6,
    /// (GMT+09:00) Osaka, Sapporo, Tokyo
    TokyoStandardTime = 0xEB,
    /// (GMT+09:00) Yakutsk
    YakutskStandardTime = 0xF0,
    /// (GMT+09:30) Darwin
    AUSCentralStandardTime = 0xF5,
    /// (GMT+09:30) Adelaide
    CenAustraliaStandardTime = 0xFA,
    /// (GMT+10:00) Canberra, Melbourne, Sydney
    AUSEasternStandardTime = 0xFF,
    /// (GMT+10:00) Brisbane
    EAustraliaStandardTime = 0x104,
    /// (GMT+10:00) Hobart
    TasmaniaStandardTime = 0x109,
    /// (GMT+10:00) Vladivostok
    VladivostokStandardTime = 0x10E,
    /// (GMT+10:00) Guam, Port Moresby
    WestPacificStandardTime = 0x113,
    /// (GMT+11:00) Magadan, Solomon Islands, New Caledonia
    CentralPacificStandardTime = 0x118,
    /// (GMT+12:00) Fiji, Kamchatka, Marshall Is.
    FijiIslandsStandardTime = 0x11D,
    /// (GMT+12:00) Auckland, Wellington
    NewZealandStandardTime = 0x122,
    /// (GMT+13:00) Nuku'alofa
    TongaStandardTime = 0x12C,
    /// (GMT-03:00) Buenos Aires
    AzerbaijanStandardTime = 0x80000040,
    /// (GMT+02:00) Beirut
    MiddleEastStandardTime = 0x80000041,
    /// (GMT+02:00) Amman
    JordanStandardTime = 0x80000042,
    /// (GMT-06:00) Guadalajara, Mexico City, Monterrey - New
    CentralStandardTime_Mexico = 0x80000043,
    /// (GMT-07:00) Chihuahua, La Paz, Mazatlan - New
    MountainStandardTime_Mexico = 0x80000044,
    /// (GMT-08:00) Tijuana, Baja California
    PacificStandardTime_Mexico = 0x80000045,
    /// (GMT+02:00) Windhoek
    NamibiaStandardTime = 0x80000046,
    /// (GMT+03:00) Tbilisi
    GeorgianStandardTime = 0x80000047,
    /// (GMT-04:00) Manaus
    CentralBrazilianStandardTime = 0x80000048,
    /// (GMT-03:00) Montevideo
    MontevideoStandardTime = 0x80000049,
    /// (GMT+04:00) Yerevan
    ArmenianStandardTime = 0x8000004A,
    /// (GMT-04:30) Caracas
    VenezuelaStandardTime = 0x8000004B,
    /// (GMT-03:00) Buenos Aires
    ArgentinaStandardTime = 0x8000004C,
    /// (GMT) Casablanca
    MoroccoStandardTime = 0x8000004D,
    /// (GMT+05:00) Islamabad, Karachi
    PakistanStandardTime = 0x8000004E,
    /// (GMT+04:00) Port Louis
    MauritiusStandardTime = 0x8000004F,
    /// (GMT) Coordinated Universal Time
    UTC = 0x80000050,
    /// (GMT-04:00) Asuncion
    ParaguayStandardTime = 0x80000051,
    /// (GMT+12:00) Petropavlovsk-Kamchatsky
    KamchatkaStandardTime = 0x80000052,
}
