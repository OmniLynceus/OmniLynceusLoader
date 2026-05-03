use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use enigo::{Coordinate, Enigo, Mouse, Settings};
use rand::{RngExt, rng};

struct ControllerState {
    target: Mutex<(i32, i32)>,
    velocity: Mutex<(f32, f32)>,
    running: AtomicBool,
}

struct ControllerConfig {
    fps: f32,
    mass: f32,
    friction: f32,
    jitter_intensity: f32,
}

pub struct Controller {
    state: Arc<ControllerState>,
    config: Arc<ControllerConfig>,
}

impl Controller {
    /// 初始化接口
    pub fn new() -> Self {
        Self {
            state: Arc::new(ControllerState {
                target: Mutex::new((500, 500)),
                velocity: Mutex::new((0.0, 0.0)),
                running: AtomicBool::new(false),
            }),
            config: Arc::new(ControllerConfig {
                fps: 144.0,
                mass: 7.8,
                friction: 0.85,
                jitter_intensity: 0.004,
            })
        }
    }

    fn friction(velocity: f32, friction: f32) -> f32 {
        let v_abs = velocity.abs();
        match v_abs {
            v if v < 50.0 => v.sqrt() * friction * velocity.signum(),
            _ => velocity * friction,
        }
    }

    fn tick(
        state: &ControllerState, 
        config: &ControllerConfig, 
        enigo: &mut Enigo, 
        rng: &mut impl RngExt
    ) {
        let (curr_x, curr_y) = enigo.location().unwrap_or((0, 0));
        let (tx, ty) = { *state.target.lock().unwrap() };
        let (dx, dy) = ((tx - curr_x) as f32, (ty - curr_y) as f32);
        let dist = (dx * dx + dy * dy).sqrt();
        

        let mut vel = state.velocity.lock().unwrap();

        let jitter = (
            rng.random_range(-config.jitter_intensity..config.jitter_intensity), 
            rng.random_range(-config.jitter_intensity..config.jitter_intensity)
        );


        let acc = (
            dx / config.mass - Self::friction(vel.0, config.friction) + jitter.0 * dist.sqrt(), 
            dy / config.mass - Self::friction(vel.1, config.friction) + jitter.1 * dist.sqrt()
        );


        vel.0 += acc.0;
        vel.1 += acc.1;


        let next_x = curr_x as f32 + vel.0;
        let next_y = curr_y as f32 + vel.1;

        enigo.move_mouse(next_x as i32, next_y as i32, Coordinate::Abs).ok(); 
    }

    fn engine(state: Arc<ControllerState>, config: Arc<ControllerConfig>) {
        let mut enigo = Enigo::new(&Settings::default()).expect("无法初始化 Enigo");
        let mut rng = rng();
    
        let frame_time = Duration::from_secs_f32(1.0 / config.fps);


        while state.running.load(Ordering::SeqCst) {
            let start = Instant::now();
            
            Self::tick(&state, &config, &mut enigo, &mut rng);

            let elapsed = start.elapsed();
            if elapsed < frame_time {
                thread::sleep(frame_time - elapsed);
            }
        }
    }

    /// 启动接口
    pub fn start(&self) {
        if self.state.running.load(Ordering::SeqCst) {
            return;
        }

        self.state.running.store(true, Ordering::SeqCst);
        let state = Arc::clone(&self.state);
        let config = Arc::clone(&self.config);

        thread::spawn(move || Self::engine(state, config));
    }

    pub fn pause(&self) {
        self.state.running.store(false, Ordering::SeqCst);
        let mut vel = self.state.velocity.lock().unwrap();
        *vel = (0.0, 0.0);
    }

    pub fn move_to(&self, x: i32, y: i32) {
        let mut target = self.state.target.lock().unwrap();
        *target = (x, y);
    }
}