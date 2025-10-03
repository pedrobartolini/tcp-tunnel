use dotenv_codegen::dotenv;

pub const PUBLIC_SERVER_HOST: &str = dotenv!("PUBLIC_SERVER_HOST");
pub const PUBLIC_SERVER_PORT: u16 = str_to_u16(dotenv!("PUBLIC_SERVER_PORT"));
pub const PRIVATE_SERVER_PORT: u16 = str_to_u16(dotenv!("PRIVATE_SERVER_PORT"));
pub const TUNNEL_LOCAL_HOST: &str = dotenv!("TUNNEL_LOCAL_HOST");
pub const SECRET_HANDSHAKE: &str = dotenv!("SECRET_HANDSHAKE");

const fn str_to_u16(s: &str) -> u16 {
    let bytes = s.as_bytes();
    let mut result: u16 = 0;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'0'..=b'9' => {
                // Check for overflow before multiplying
                if result > u16::MAX / 10 {
                    panic!("Port number too large");
                }
                result = result * 10;

                // Check for overflow before adding
                if result > u16::MAX - (bytes[i] - b'0') as u16 {
                    panic!("Port number too large");
                }
                result = result + (bytes[i] - b'0') as u16;
            }
            _ => panic!("Invalid character in port number")
        }
        i += 1;
    }

    if i == 0 {
        panic!("Empty port string");
    }

    // Validate port range (0-65535)
    if result == 0 {
        panic!("Port cannot be 0");
    }

    result
}
