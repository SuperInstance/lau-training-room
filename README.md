# lau-training-room

> Curriculum-driven training rooms for the PLATO agent ecosystem — skill tracking, mentorship, peer observation, and graduation in a single Rust library.

## What This Does

`lau-training-room` provides the infrastructure for structured agent training inside the PLATO ecosystem. It models **training rooms** where agents enroll in **curricula**, practice **skills**, learn from **mentors** and **peers**, and **graduate** once all skill objectives reach mastery. An academy layer manages multiple rooms, tracks agent histories, and reports aggregate statistics.

Think of it as a school system for AI agents: curricula define what needs to be learned, rooms are the classrooms, and the academy is the institution.

---

## Key Idea

Agents don't just "know" things — they *learn*. This library formalizes that process:

1. **Curricula** define ordered skill objectives with mastery thresholds and evaluation methods.
2. **Training Rooms** host agents working through a curriculum, tracking per-skill progress and attempt counts.
3. **Mentorship & Observation** let agents learn from each other — observing a peer's mastery grants 30% of their progress in that skill.
4. **Graduation** happens automatically when every skill in the curriculum reaches mastery (≥ 0.99).
5. **Academy** manages multiple rooms and curricula, tracking which agents have been where.

---

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-training-room = "0.1"
```

Requires **Rust 2021 edition** or later.

---

## Quick Start

```rust
use lau_training_room::*;

// 1. Create an academy with curricula
let mut academy = TrainingAcademy::new();
academy.add_curriculum(conservation_basics());
academy.add_curriculum(builder_fundamentals());

// 2. Open a training room
let room_id = academy.create_room("conservation-basics").unwrap();

// 3. Enroll agents
academy.enroll(&room_id, "alice".to_string());
academy.enroll(&room_id, "bob".to_string());

// 4. Assign a mentor
let room = academy.rooms.get_mut(&room_id).unwrap();
room.set_mentor("professor_x".to_string());

// 5. Record skill attempts
room.record_attempt("alice", "check_balance", 0.85);
room.record_attempt("alice", "check_balance", 1.0);
room.record_attempt("alice", "verify_total", 1.0);
room.record_attempt("alice", "correct_error", 1.0);

// 6. Bob observes Alice's mastery
room.record_observation("bob", "alice", "check_balance");

// 7. Check graduation
if room.check_graduation("alice") {
    println!("🎉 Alice graduated!");
}

// 8. Academy-wide stats
let stats = academy.academy_stats();
println!("Rooms: {}, Enrollments: {}, Graduations: {}",
    stats.total_rooms, stats.total_enrollments, stats.total_graduations);
```

---

## API Reference

### Core Types

| Type | Description |
|------|-------------|
| `TrainingRoomId(String)` | Newtype wrapper for room identifiers. Implements `Hash`, `Eq`, `Display`. |
| `TrainingCategory` | Enum: `Building`, `Farming`, `AgentComposition`, `ResourceManagement`, `Scouting`, `Trading`, `Conservation`, `Leadership`, `Adaptation`. |
| `EvalMethod` | How a skill is evaluated: `ConservationAccuracy`, `TaskCompletion`, `PeerComparison`, `BehavioralMatch`. |
| `SkillObjective` | A single skill with `skill_id`, `description`, `mastery_threshold`, and `evaluation_method`. |
| `Curriculum` | A named collection of skills with a `difficulty` level, `category`, and `prerequisites`. |
| `TraineeState` | Per-agent state: `skill_progress` map, `attempts` counter, `mentor_id`, `graduated` flag. |
| `TrainingEvent` | Event log enum: `AgentEntered`, `AgentGraduated`, `SkillAttempted`, `MentorAssigned`, `ObservationMade`, `CollaborationStarted`, `RoomCompleted`. |
| `TrainingRoom` | A room hosting trainees working through a single curriculum. |
| `AcademyStats` | Aggregate: `total_rooms`, `total_enrollments`, `total_graduations`, `curricula_offered`, `active_rooms`. |
| `TrainingAcademy` | Top-level manager: curricula registry, room pool, agent history. |

### `TraineeState`

```rust
let progress = trainee.progress_on("check_balance"); // 0.0 if not started
let mastered = trainee.is_mastered("check_balance");  // true if ≥ 0.99
let overall = trainee.overall_progress();             // mean across all skills
trainee.attempt("mining");                            // increment attempt counter
trainee.update_skill("mining", 0.75);                 // set progress (clamped to [0, 1])
```

### `TrainingRoom`

| Method | Description |
|--------|-------------|
| `new(id, curriculum, max_trainees)` | Create a room. |
| `set_mentor(mentor_id)` | Assign a mentor to all current and future trainees. |
| `enter(agent_id) → bool` | Enroll an agent (fails if room is full). |
| `leave(agent_id) → Option<TraineeState>` | Remove an agent, returning their state. |
| `record_attempt(agent, skill, score)` | Record a skill attempt (score clamped to [0, 1]). |
| `record_observation(observer, target, skill)` | Observer gains 30% of target's mastery in the skill. |
| `check_graduation(agent) → bool` | Check if agent mastered all skills; increments graduation count. |
| `graduated_agents() → Vec<&TraineeState>` | Agents who have graduated. |
| `active_agents() → Vec<&TraineeState>` | Agents still training. |
| `room_progress() → f64` | Mean progress across all trainees (0.0–1.0). |
| `events() → &[TrainingEvent]` | Read-only event log. |
| `summary() → String` | Human-readable room status. |

### `TrainingAcademy`

| Method | Description |
|--------|-------------|
| `new()` | Empty academy. |
| `add_curriculum(curriculum)` | Register a curriculum by its `id`. |
| `create_room(curriculum_name) → Option<TrainingRoomId>` | Open a new room for a curriculum. |
| `enroll(room_id, agent_id) → bool` | Enroll an agent into a room (tracks history). |
| `agent_rooms(agent_id) → Vec<&TrainingRoom>` | All rooms an agent has ever been in. |
| `academy_stats() → AcademyStats` | Aggregate statistics. |

### Pre-built Curricula

The library ships five ready-to-use curricula:

| Function | Skills | Difficulty | Prerequisites |
|----------|--------|------------|---------------|
| `conservation_basics()` | 3 (check_balance, verify_total, correct_error) | 0.2 | None |
| `builder_fundamentals()` | 4 (place_blocks, read_blueprint, conserve_materials, structural_integrity) | 0.3 | None |
| `agent_composition_101()` | 3 (identify_capabilities, compose_masks, verify_composition) | 0.5 | builder_fundamentals |
| `advanced_farming()` | 5 (soil_chemistry, crop_rotation, water_management, pest_control, yield_optimization) | 0.7 | conservation_basics |
| `leadership_training()` | 4 (assign_tasks, evaluate_performance, compose_teams, handle_failure) | 0.9 | agent_composition_101, builder_fundamentals |

---

## How It Works

### Room Lifecycle

```
Create Academy → Register Curricula → Open Rooms → Enroll Agents
                                                       ↓
                                              Record Attempts
                                              Record Observations
                                              Check Graduation
                                                       ↓
                                              Academy Stats / Reports
```

### Observation Learning

When agent B observes agent A performing a skill:

```rust
room.record_observation("bob", "alice", "check_balance");
```

Bob gains `alice.progress_on("check_balance") * 0.3` in that skill. This models the idea that watching an expert accelerates learning, but doesn't replace practice.

### Serialization

All types derive `Serialize`/`Deserialize` via serde. Round-trip serialization is tested for every major type.

---

## The Math

The library is primarily algorithmic rather than mathematical, but the key formulas are:

**Skill Progress Update:**
```
progress_new = clamp(score, 0.0, 1.0)
```
Scores are clamped rather than accumulated — the last attempt's score *is* the current progress.

**Observation Gain:**
```
gain = target_mastery × 0.3
progress_new = clamp(current + gain, 0.0, 1.0)
```

**Overall Progress (per trainee):**
```
overall = (1/n) × Σ progress_i   over all skills i in the curriculum
```

**Room Progress:**
```
room_progress = (1/m) × Σ overall_j   over all trainees j
```

**Graduation Condition:**
```
∀ skills s in curriculum: progress(s) ≥ 0.99
```

---

## Tests

54 unit tests covering:

- Training room ID equality, hashing, cloning, display
- Trainee state: progress tracking, mastery checks, attempt counting, clamping
- Room lifecycle: enter, leave, capacity enforcement, event logging
- Mentorship propagation
- Peer observation learning
- Graduation (single, multiple, edge cases)
- Academy: curriculum registration, room creation, enrollment, history tracking, stats
- All 5 pre-built curricula structure validation
- Serde round-trip serialization for all major types

Run with:
```bash
cargo test
```

---

## License

MIT
