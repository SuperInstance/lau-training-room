# lau-training-room

An A2A training academy for agents. Five curricula, observation learning, and evaluation — because agents need to learn before they deploy.

## The concept in 60 seconds

Before an ensign takes a shift, it trains in the training room. This crate implements:

- **Training rooms** with configurable curricula and evaluation methods
- **Five curricula:** basic protocols, advanced reasoning, crisis response, cultural competence, specialist skills
- **Observation learning:** agents learn by watching other agents (not just by being told)
- **Evaluation:** pass/fail with scoring, not just completion
- **Graduation:** agents that pass all required curricula get certified for deployment

## Quick start

```rust
use lau_training_room::{TrainingRoom, Curriculum, Trainee, EvalMethod};

let mut room = TrainingRoom::new("academy_alpha");

// Add a curriculum
room.add_curriculum(Curriculum::crisis_response()
    .with_passing_score(0.8)
    .with_eval(EvalMethod::Scenario));

// Enroll a trainee
let trainee = Trainee::new("ensign_7").with_background("general");
room.enroll(trainee);

// Run training session
let results = room.train("ensign_7", "crisis_response");
if results.passed() {
    println!("{} graduated with score {:.2}", results.trainee, results.score);
}
```

## Contributing

[Open an issue](https://github.com/SuperInstance/lau-training-room/issues) or PR.
