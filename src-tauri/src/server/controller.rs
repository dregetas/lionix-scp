pub trait ServerController {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn send_cmd(&mut self, cmd: &str) -> Result<(), String>;
}
