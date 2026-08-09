pub fn transform_id(id: u8) -> &'static str {
  match id {
    0x00 => "system:log",
    0x01 => "system:message",
    0x02 => "system:connected-bots",
    0x03 => "system:total-bots",
    0x04 => "session:chat",
    0x05 => "monitoring:chat",
    0x06 => "captcha:new-web",
    0x07 => "captcha:new-map",
    0x08 => "captcha:remove",
    0x09 => "status:launch",
    0x0A => "status:stop",
    0x0B => "monitoring:profiles",
    0x0C => "process:display-status",
    0x0D => "radar:target-info",
    0x0E => "process:synchronize",
    _ => "unknown",
  }
}
