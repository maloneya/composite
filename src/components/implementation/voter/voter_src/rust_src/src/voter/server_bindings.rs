use voter::voter_config::BUFF_SIZE;

pub fn handle_request(serialized_msg: [u8; BUFF_SIZE]) -> [u8; BUFF_SIZE] {
    let sinv_id = serialized_msg[0];

    match sinv_id {
        0 => [rk_write(&serialized_msg[1..]); BUFF_SIZE],
        1 => rk_read(&serialized_msg[1..]),
        _ => panic!("sinv_id {:?} not supported", sinv_id),
    }
}

fn rk_write(data: &[u8]) -> u8 {
    println!("wrote {:?}", data);
    data.len() as u8
}

fn rk_read(_data: &[u8]) -> [u8; BUFF_SIZE] {
    println!("Read");
    [12; BUFF_SIZE]
}
