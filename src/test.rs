#[cfg(test)]
mod tests {
    use std::net::{ IpAddr, Ipv4Addr };

    use chrono::naive::NaiveDate;
    use chrono::offset::{ FixedOffset, TimeZone };
    use chrono::prelude::DateTime;
    use packed_struct::prelude::PackedStruct;

    use crate::{
        constants,
        network::{
            AuthenticationMessage,
            CommandMessage,
            DiscoveryMessage,
            RemoteDataCommand,
            RemoteDataMessage,
            WirelessConnection,
        },
    };

    #[test]
    fn authentication_packs_correctly() {
        let auth = AuthenticationMessage::new("Test 1");

        // Calculated using the python-broadlink library
        let expected: [u8; 0x50] = [0,0,0,0,49,49,49,49,49,49,49,49,49,49,49,49,49,49,49,49,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,84,101,115,116,32,49,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        let actual = auth.pack()
            .expect("Could not pack test auth message!");

        assert_eq!(expected, actual);
    }

    #[test]
    fn command_packs_correctly() {
        let payload: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cmd = CommandMessage::with_count::<AuthenticationMessage>(
            0x1234,
            0x649B,
            [0x1u8, 0x2u8, 0x3u8, 0x4u8, 0x5u8, 0x6u8],
            0xABCDEFAB,
        );

        // Calculated using the python-broadlink library
        let expected: &[u8] = &[90,165,170,85,90,165,170,85,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,205,209,0,0,155,100,101,0,52,146,6,5,4,3,2,1,171,239,205,171,220,190,0,0,165,197,88,183,43,70,174,88,109,241,187,8,228,74,30,218];
        let actual = cmd.pack_with_payload(&payload, &constants::INITIAL_KEY)
            .expect("Could not pack test command message!");

        assert_eq!(expected, &actual);
    }

    #[test]
    fn discovery_packs_correctly() {
        // Note: No idea why we must +1 on the minute, but this test will fail otherwise
        let discover = DiscoveryMessage::new(
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            42424,
            Some(DateTime::from_utc(
                NaiveDate::from_ymd(2000, 2, 14).and_hms(10, 30 + 1, 0),
                TimeZone::from_offset(&FixedOffset::west(5)),
            )),
        ).expect("Could not construct DiscoveryMessage!");

        // Calculated using the python-broadlink library
        let expected: [u8; 48] = [0,0,0,0,0,0,0,0,251,255,255,255,208,7,30,10,0,1,14,2,0,0,0,0,4,3,2,1,184,165,0,0,36,197,0,0,0,0,6,0,0,0,0,0,0,0,0,0];
        let actual = discover.pack()
            .expect("Could not pack test discover message!");

        assert_eq!(expected, actual);
    }

    #[test]
    fn remote_data_packs_correctly() {
        let remote = RemoteDataMessage::new(RemoteDataCommand::SendCode);
        let payload: [u8; 8] = [0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89];

        // Calculated using the python-broadlink library
        let expected: &[u8] = &[12, 0, 2, 0, 0, 0, 171, 205, 239, 1, 35, 69, 103, 137];
        let actual = remote.pack_with_payload(&payload)
            .expect("Could not pack test remote data message!");

        assert_eq!(expected, &actual);
    }

    #[test]
    fn wireless_connection_packs_correctly() {
        let connection = WirelessConnection::WPA1(
            "Test SSID",
            "Test Password",
        );

        // Calculated using the python-broadlink library
        let expected: [u8; 136] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,225,198,0,0,0,0,20,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,84,101,115,116,32,83,83,73,68,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,84,101,115,116,32,80,97,115,115,119,111,114,100,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,13,2,0];
        let actual = connection
            .to_message()
            .expect("Could not create test connection message!")
            .pack()
            .expect("Could not pack test connection message!");

        assert_eq!(expected, actual);
    }
}
