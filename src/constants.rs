use hex_literal::hex;

/// The initial key used by broadlink devices before authentication.
pub const INITIAL_KEY: [u8; 16] = hex!("097628343fe99e23765c1513accf8b02");

/// The initial IV used by broadlink devices for all authentication requests.
pub const INITIAL_VECTOR: [u8; 16] = hex!("562e17996d093d28ddb3ba695a2e6f58");
