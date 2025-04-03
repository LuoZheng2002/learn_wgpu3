use std::time::Instant;



pub struct State{
    // accumulated time
    pub timer: Option<Instant>,
    pub prev_time: Option<f32>,
    pub fps_timer: Option<Instant>,
    pub accumulated_frame_num: u32,
}


impl State{
    pub fn update(&mut self){
        // calculate fps every 1 second
        let fps_timer = self.fps_timer.get_or_insert_with(||Instant::now());
        let current_time = fps_timer.elapsed().as_secs_f32();
        if current_time >= 1.0 {
            println!("FPS: {}", self.accumulated_frame_num);
            self.accumulated_frame_num = 0;
            *fps_timer = Instant::now();
        } else {
            self.accumulated_frame_num += 1;
        }
        let timer = self.timer.get_or_insert_with(||Instant::now());
        let current_time = timer.elapsed().as_secs_f32();
        let prev_time = self.prev_time.get_or_insert(current_time);
        let delta_time = current_time - *prev_time;
        assert!(delta_time >= 0.0);
        *prev_time = current_time;
    }
}


impl Default for State {
    fn default() -> Self {
        State {
            timer: None,
            prev_time: None,
            fps_timer: None,
            accumulated_frame_num: 0,
        }
    }
}