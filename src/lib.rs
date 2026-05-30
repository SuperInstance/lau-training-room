use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TrainingRoomId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrainingRoomId(pub String);

impl std::fmt::Display for TrainingRoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// EvalMethod
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvalMethod {
    ConservationAccuracy { error: f64, threshold: f64 },
    TaskCompletion { time_limit_secs: f64 },
    PeerComparison,
    BehavioralMatch { mentor_pattern: String },
}

impl EvalMethod {
    pub fn name(&self) -> &str {
        match self {
            EvalMethod::ConservationAccuracy { .. } => "conservation_accuracy",
            EvalMethod::TaskCompletion { .. } => "task_completion",
            EvalMethod::PeerComparison => "peer_comparison",
            EvalMethod::BehavioralMatch { .. } => "behavioral_match",
        }
    }
}

// ---------------------------------------------------------------------------
// SkillObjective
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillObjective {
    pub skill_id: String,
    pub description: String,
    pub mastery_threshold: f64,
    pub evaluation_method: EvalMethod,
}

// ---------------------------------------------------------------------------
// TrainingCategory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingCategory {
    Building,
    Farming,
    AgentComposition,
    ResourceManagement,
    Scouting,
    Trading,
    Conservation,
    Leadership,
    Adaptation,
}

impl std::fmt::Display for TrainingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrainingCategory::Building => write!(f, "Building"),
            TrainingCategory::Farming => write!(f, "Farming"),
            TrainingCategory::AgentComposition => write!(f, "AgentComposition"),
            TrainingCategory::ResourceManagement => write!(f, "ResourceManagement"),
            TrainingCategory::Scouting => write!(f, "Scouting"),
            TrainingCategory::Trading => write!(f, "Trading"),
            TrainingCategory::Conservation => write!(f, "Conservation"),
            TrainingCategory::Leadership => write!(f, "Leadership"),
            TrainingCategory::Adaptation => write!(f, "Adaptation"),
        }
    }
}

// ---------------------------------------------------------------------------
// Curriculum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curriculum {
    pub id: String,
    pub name: String,
    pub skills: Vec<SkillObjective>,
    pub difficulty: f64,
    pub category: TrainingCategory,
    pub prerequisites: Vec<String>,
}

// ---------------------------------------------------------------------------
// TraineeState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraineeState {
    pub agent_id: String,
    pub skill_progress: HashMap<String, f64>,
    pub attempts: HashMap<String, u32>,
    pub mentor_id: Option<String>,
    pub room_id: TrainingRoomId,
    pub entered_tick: u64,
    pub graduated: bool,
}

impl TraineeState {
    pub fn progress_on(&self, skill: &str) -> f64 {
        self.skill_progress.get(skill).copied().unwrap_or(0.0)
    }

    pub fn is_mastered(&self, skill: &str) -> bool {
        self.skill_progress.get(skill).copied().unwrap_or(0.0) >= 0.99
    }

    pub fn overall_progress(&self) -> f64 {
        if self.skill_progress.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.skill_progress.values().sum();
        sum / self.skill_progress.len() as f64
    }

    pub fn attempt(&mut self, skill: &str) {
        *self.attempts.entry(skill.to_string()).or_insert(0) += 1;
    }

    pub fn update_skill(&mut self, skill: &str, progress: f64) {
        let clamped = progress.clamp(0.0, 1.0);
        self.skill_progress.insert(skill.to_string(), clamped);
    }
}

// ---------------------------------------------------------------------------
// TrainingEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingEvent {
    AgentEntered(String),
    AgentGraduated(String),
    SkillAttempted { agent: String, skill: String, score: f64 },
    MentorAssigned { trainee: String, mentor: String },
    ObservationMade { observer: String, target: String, learned: String },
    CollaborationStarted(Vec<String>),
    RoomCompleted,
}

// ---------------------------------------------------------------------------
// TrainingRoom
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRoom {
    pub id: TrainingRoomId,
    pub curriculum: Curriculum,
    pub trainees: HashMap<String, TraineeState>,
    pub mentor_id: Option<String>,
    pub max_trainees: usize,
    pub tick: u64,
    events: Vec<TrainingEvent>,
    pub graduation_count: u32,
}

impl TrainingRoom {
    pub fn new(id: &str, curriculum: Curriculum, max_trainees: usize) -> Self {
        TrainingRoom {
            id: TrainingRoomId(id.to_string()),
            curriculum,
            trainees: HashMap::new(),
            mentor_id: None,
            max_trainees,
            tick: 0,
            events: Vec::new(),
            graduation_count: 0,
        }
    }

    pub fn set_mentor(&mut self, mentor_id: String) {
        self.mentor_id = Some(mentor_id.clone());
    }

    pub fn enter(&mut self, agent_id: String) -> bool {
        if self.trainees.len() >= self.max_trainees {
            return false;
        }
        let skill_progress: HashMap<String, f64> = self
            .curriculum
            .skills
            .iter()
            .map(|s| (s.skill_id.clone(), 0.0))
            .collect();
        let state = TraineeState {
            agent_id: agent_id.clone(),
            skill_progress,
            attempts: HashMap::new(),
            mentor_id: self.mentor_id.clone(),
            room_id: self.id.clone(),
            entered_tick: self.tick,
            graduated: false,
        };
        self.trainees.insert(agent_id.clone(), state);
        self.events.push(TrainingEvent::AgentEntered(agent_id));
        true
    }

    pub fn leave(&mut self, agent_id: String) -> Option<TraineeState> {
        self.trainees.remove(&agent_id)
    }

    pub fn record_attempt(&mut self, agent_id: &str, skill: &str, score: f64) {
        if let Some(trainee) = self.trainees.get_mut(agent_id) {
            trainee.attempt(skill);
            let clamped = score.clamp(0.0, 1.0);
            trainee.update_skill(skill, clamped);
            self.events.push(TrainingEvent::SkillAttempted {
                agent: agent_id.to_string(),
                skill: skill.to_string(),
                score: clamped,
            });
        }
    }

    pub fn record_observation(&mut self, observer: &str, target: &str, learned_skill: &str) {
        if let Some(target_state) = self.trainees.get(target) {
            let target_mastery = target_state.progress_on(learned_skill);
            if let Some(obs) = self.trainees.get_mut(observer) {
                let gain = target_mastery * 0.3;
                let current = obs.progress_on(learned_skill);
                let new_val = (current + gain).clamp(0.0, 1.0);
                obs.update_skill(learned_skill, new_val);
                self.events.push(TrainingEvent::ObservationMade {
                    observer: observer.to_string(),
                    target: target.to_string(),
                    learned: learned_skill.to_string(),
                });
            }
        }
    }

    pub fn check_graduation(&mut self, agent_id: &str) -> bool {
        let mastered = self
            .trainees
            .get(agent_id)
            .map(|t| {
                self.curriculum
                    .skills
                    .iter()
                    .all(|s| t.is_mastered(&s.skill_id))
            })
            .unwrap_or(false);

        if mastered {
            if let Some(t) = self.trainees.get_mut(agent_id) {
                t.graduated = true;
            }
            self.graduation_count += 1;
            self.events
                .push(TrainingEvent::AgentGraduated(agent_id.to_string()));
        }
        mastered
    }

    pub fn graduated_agents(&self) -> Vec<&TraineeState> {
        self.trainees.values().filter(|t| t.graduated).collect()
    }

    pub fn active_agents(&self) -> Vec<&TraineeState> {
        self.trainees.values().filter(|t| !t.graduated).collect()
    }

    pub fn room_progress(&self) -> f64 {
        if self.trainees.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.trainees.values().map(|t| t.overall_progress()).sum();
        sum / self.trainees.len() as f64
    }

    pub fn events(&self) -> &[TrainingEvent] {
        &self.events
    }

    pub fn summary(&self) -> String {
        let total = self.trainees.len();
        let graduated = self.graduated_agents().len();
        let active = total - graduated;
        let room_progress = self.room_progress();
        format!(
            "Room {} | Curriculum: {} | Trainees: {} total ({} active, {} graduated) | Room progress: {:.1}% | Mentor: {}",
            self.id,
            self.curriculum.name,
            total,
            active,
            graduated,
            room_progress * 100.0,
            self.mentor_id.as_deref().unwrap_or("none")
        )
    }
}

// ---------------------------------------------------------------------------
// AcademyStats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademyStats {
    pub total_rooms: usize,
    pub total_enrollments: usize,
    pub total_graduations: usize,
    pub curricula_offered: usize,
    pub active_rooms: usize,
}

// ---------------------------------------------------------------------------
// TrainingAcademy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingAcademy {
    pub rooms: HashMap<TrainingRoomId, TrainingRoom>,
    pub curricula: HashMap<String, Curriculum>,
    pub agent_history: HashMap<String, Vec<TrainingRoomId>>,
}

impl TrainingAcademy {
    pub fn new() -> Self {
        TrainingAcademy {
            rooms: HashMap::new(),
            curricula: HashMap::new(),
            agent_history: HashMap::new(),
        }
    }

    pub fn add_curriculum(&mut self, curriculum: Curriculum) {
        self.curricula.insert(curriculum.id.clone(), curriculum);
    }

    pub fn create_room(&mut self, curriculum_name: &str) -> Option<TrainingRoomId> {
        let curriculum = self.curricula.get(curriculum_name)?.clone();
        let room_id_str = format!("{}-{}", curriculum_name, self.rooms.len() + 1);
        let room_id = TrainingRoomId(room_id_str.clone());
        let room = TrainingRoom::new(&room_id_str, curriculum, 10);
        self.rooms.insert(room_id.clone(), room);
        Some(room_id)
    }

    pub fn enroll(&mut self, room_id: &TrainingRoomId, agent_id: String) -> bool {
        let ok = self
            .rooms
            .get_mut(room_id)
            .map(|room| room.enter(agent_id.clone()))
            .unwrap_or(false);
        if ok {
            self.agent_history
                .entry(agent_id)
                .or_default()
                .push(room_id.clone());
        }
        ok
    }

    pub fn agent_rooms(&self, agent_id: &str) -> Vec<&TrainingRoom> {
        self.agent_history
            .get(agent_id)
            .map(|ids| ids.iter().filter_map(|rid| self.rooms.get(rid)).collect())
            .unwrap_or_default()
    }

    pub fn academy_stats(&self) -> AcademyStats {
        let total_rooms = self.rooms.len();
        let total_enrollments: usize = self.rooms.values().map(|r| r.trainees.len()).sum();
        let total_graduations: u32 = self.rooms.values().map(|r| r.graduation_count).sum();
        let curricula_offered = self.curricula.len();
        let active_rooms = self
            .rooms
            .values()
            .filter(|r| !r.trainees.is_empty())
            .count();
        AcademyStats {
            total_rooms,
            total_enrollments,
            total_graduations: total_graduations as usize,
            curricula_offered,
            active_rooms,
        }
    }
}

impl Default for TrainingAcademy {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pre-built curricula
// ---------------------------------------------------------------------------

pub fn conservation_basics() -> Curriculum {
    Curriculum {
        id: "conservation-basics".to_string(),
        name: "Conservation Basics".to_string(),
        skills: vec![
            SkillObjective {
                skill_id: "check_balance".to_string(),
                description: "Check item balance in inventories".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::ConservationAccuracy {
                    error: 0.05,
                    threshold: 0.1,
                },
            },
            SkillObjective {
                skill_id: "verify_total".to_string(),
                description: "Verify total item counts".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::ConservationAccuracy {
                    error: 0.03,
                    threshold: 0.1,
                },
            },
            SkillObjective {
                skill_id: "correct_error".to_string(),
                description: "Detect and correct inventory discrepancies".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::TaskCompletion {
                    time_limit_secs: 60.0,
                },
            },
        ],
        difficulty: 0.2,
        category: TrainingCategory::Conservation,
        prerequisites: vec![],
    }
}

pub fn builder_fundamentals() -> Curriculum {
    Curriculum {
        id: "builder-fundamentals".to_string(),
        name: "Builder Fundamentals".to_string(),
        skills: vec![
            SkillObjective {
                skill_id: "place_blocks".to_string(),
                description: "Place blocks accurately according to plan".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::TaskCompletion {
                    time_limit_secs: 30.0,
                },
            },
            SkillObjective {
                skill_id: "read_blueprint".to_string(),
                description: "Read and interpret building blueprints".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::BehavioralMatch {
                    mentor_pattern: "blueprint_reader".to_string(),
                },
            },
            SkillObjective {
                skill_id: "conserve_materials".to_string(),
                description: "Minimize material waste during construction".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::TaskCompletion {
                    time_limit_secs: 45.0,
                },
            },
            SkillObjective {
                skill_id: "structural_integrity".to_string(),
                description: "Ensure structures meet stability requirements".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::PeerComparison,
            },
        ],
        difficulty: 0.3,
        category: TrainingCategory::Building,
        prerequisites: vec![],
    }
}

pub fn agent_composition_101() -> Curriculum {
    Curriculum {
        id: "agent-composition-101".to_string(),
        name: "Agent Composition 101".to_string(),
        skills: vec![
            SkillObjective {
                skill_id: "identify_capabilities".to_string(),
                description: "Identify agent capabilities from their configuration".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::ConservationAccuracy {
                    error: 0.1,
                    threshold: 0.15,
                },
            },
            SkillObjective {
                skill_id: "compose_masks".to_string(),
                description: "Compose capability masks for complex tasks".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::BehavioralMatch {
                    mentor_pattern: "mask_composer".to_string(),
                },
            },
            SkillObjective {
                skill_id: "verify_composition".to_string(),
                description: "Verify that agent compositions are valid".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::TaskCompletion {
                    time_limit_secs: 90.0,
                },
            },
        ],
        difficulty: 0.5,
        category: TrainingCategory::AgentComposition,
        prerequisites: vec!["builder-fundamentals".to_string()],
    }
}

pub fn advanced_farming() -> Curriculum {
    Curriculum {
        id: "advanced-farming".to_string(),
        name: "Advanced Farming".to_string(),
        skills: vec![
            SkillObjective {
                skill_id: "soil_chemistry".to_string(),
                description: "Analyze and adjust soil chemistry for optimal growth".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::ConservationAccuracy {
                    error: 0.02,
                    threshold: 0.05,
                },
            },
            SkillObjective {
                skill_id: "crop_rotation".to_string(),
                description: "Plan and execute crop rotation schedules".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::TaskCompletion {
                    time_limit_secs: 120.0,
                },
            },
            SkillObjective {
                skill_id: "water_management".to_string(),
                description: "Manage irrigation and water distribution".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::PeerComparison,
            },
            SkillObjective {
                skill_id: "pest_control".to_string(),
                description: "Identify and control pests without damaging crops".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::ConservationAccuracy {
                    error: 0.05,
                    threshold: 0.1,
                },
            },
            SkillObjective {
                skill_id: "yield_optimization".to_string(),
                description: "Optimize crop yield through integrated techniques".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::BehavioralMatch {
                    mentor_pattern: "master_farmer".to_string(),
                },
            },
        ],
        difficulty: 0.7,
        category: TrainingCategory::Farming,
        prerequisites: vec!["conservation-basics".to_string()],
    }
}

pub fn leadership_training() -> Curriculum {
    Curriculum {
        id: "leadership-training".to_string(),
        name: "Leadership Training".to_string(),
        skills: vec![
            SkillObjective {
                skill_id: "assign_tasks".to_string(),
                description: "Assign tasks optimally to team members".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::PeerComparison,
            },
            SkillObjective {
                skill_id: "evaluate_performance".to_string(),
                description: "Evaluate agent performance and give feedback".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::ConservationAccuracy {
                    error: 0.1,
                    threshold: 0.15,
                },
            },
            SkillObjective {
                skill_id: "compose_teams".to_string(),
                description: "Compose effective teams from available agents".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::BehavioralMatch {
                    mentor_pattern: "team_lead".to_string(),
                },
            },
            SkillObjective {
                skill_id: "handle_failure".to_string(),
                description: "Respond to and recover from team failures".to_string(),
                mastery_threshold: 0.99,
                evaluation_method: EvalMethod::TaskCompletion {
                    time_limit_secs: 180.0,
                },
            },
        ],
        difficulty: 0.9,
        category: TrainingCategory::Leadership,
        prerequisites: vec![
            "agent-composition-101".to_string(),
            "builder-fundamentals".to_string(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- TrainingRoomId ---

    #[test]
    fn test_training_room_id_newtype() {
        let id1 = TrainingRoomId("r1".to_string());
        let id2 = TrainingRoomId("r1".to_string());
        let id3 = TrainingRoomId("r2".to_string());
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.to_string(), "r1");
    }

    #[test]
    fn test_training_room_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TrainingRoomId("a".to_string()));
        set.insert(TrainingRoomId("a".to_string()));
        set.insert(TrainingRoomId("b".to_string()));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_training_room_id_clone() {
        let id = TrainingRoomId("test-room".to_string());
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    // --- TraineeState ---

    #[test]
    fn test_trainee_progress_on() {
        let room_id = TrainingRoomId("r1".to_string());
        let mut progress = HashMap::new();
        progress.insert("crop_rotation".to_string(), 0.75);
        let trainee = TraineeState {
            agent_id: "agent1".to_string(),
            skill_progress: progress,
            attempts: HashMap::new(),
            mentor_id: None,
            room_id,
            entered_tick: 0,
            graduated: false,
        };
        assert_eq!(trainee.progress_on("crop_rotation"), 0.75);
        assert_eq!(trainee.progress_on("unknown"), 0.0);
    }

    #[test]
    fn test_trainee_is_mastered() {
        let room_id = TrainingRoomId("r1".to_string());
        let mut progress = HashMap::new();
        progress.insert("skill_a".to_string(), 1.0);
        progress.insert("skill_b".to_string(), 0.5);
        let trainee = TraineeState {
            agent_id: "agent2".to_string(),
            skill_progress: progress,
            attempts: HashMap::new(),
            mentor_id: None,
            room_id,
            entered_tick: 0,
            graduated: false,
        };
        assert!(trainee.is_mastered("skill_a"));
        assert!(!trainee.is_mastered("skill_b"));
    }

    #[test]
    fn test_trainee_overall_progress() {
        let room_id = TrainingRoomId("r1".to_string());
        let mut progress = HashMap::new();
        progress.insert("x".to_string(), 1.0);
        progress.insert("y".to_string(), 0.5);
        let trainee = TraineeState {
            agent_id: "agent3".to_string(),
            skill_progress: progress,
            attempts: HashMap::new(),
            mentor_id: None,
            room_id,
            entered_tick: 0,
            graduated: false,
        };
        assert!((trainee.overall_progress() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_trainee_overall_progress_empty() {
        let room_id = TrainingRoomId("r1".to_string());
        let trainee = TraineeState {
            agent_id: "empty".to_string(),
            skill_progress: HashMap::new(),
            attempts: HashMap::new(),
            mentor_id: None,
            room_id,
            entered_tick: 0,
            graduated: false,
        };
        assert_eq!(trainee.overall_progress(), 0.0);
    }

    #[test]
    fn test_trainee_attempt_increments() {
        let room_id = TrainingRoomId("r1".to_string());
        let mut trainee = TraineeState {
            agent_id: "agent5".to_string(),
            skill_progress: HashMap::new(),
            attempts: HashMap::new(),
            mentor_id: None,
            room_id,
            entered_tick: 0,
            graduated: false,
        };
        trainee.attempt("mining");
        trainee.attempt("mining");
        trainee.attempt("crafting");
        assert_eq!(trainee.attempts.get("mining"), Some(&2));
        assert_eq!(trainee.attempts.get("crafting"), Some(&1));
    }

    #[test]
    fn test_trainee_update_skill_clamps() {
        let room_id = TrainingRoomId("r1".to_string());
        let mut trainee = TraineeState {
            agent_id: "agent6".to_string(),
            skill_progress: HashMap::new(),
            attempts: HashMap::new(),
            mentor_id: None,
            room_id,
            entered_tick: 0,
            graduated: false,
        };
        trainee.update_skill("over", 1.5);
        trainee.update_skill("under", -0.5);
        assert!((trainee.progress_on("over") - 1.0).abs() < 1e-9);
        assert!((trainee.progress_on("under") - 0.0).abs() < 1e-9);
    }

    // --- TrainingRoom ---

    #[test]
    fn test_training_room_new() {
        let room = TrainingRoom::new("room1", builder_fundamentals(), 5);
        assert_eq!(room.id.to_string(), "room1");
        assert_eq!(room.max_trainees, 5);
        assert!(room.mentor_id.is_none());
        assert!(room.trainees.is_empty());
    }

    #[test]
    fn test_set_mentor() {
        let mut room = TrainingRoom::new("r2", conservation_basics(), 3);
        room.set_mentor("mentor-prime".to_string());
        assert_eq!(room.mentor_id, Some("mentor-prime".to_string()));
    }

    #[test]
    fn test_enter_and_leave() {
        let mut room = TrainingRoom::new("r3", builder_fundamentals(), 5);
        assert!(room.enter("alice".to_string()));
        assert!(room.enter("bob".to_string()));
        assert_eq!(room.trainees.len(), 2);

        let left = room.leave("alice".to_string());
        assert!(left.is_some());
        assert_eq!(left.unwrap().agent_id, "alice");
        assert_eq!(room.trainees.len(), 1);
    }

    #[test]
    fn test_enter_when_full() {
        let mut room = TrainingRoom::new("r4", builder_fundamentals(), 1);
        assert!(room.enter("alice".to_string()));
        assert!(!room.enter("bob".to_string()));
        assert_eq!(room.trainees.len(), 1);
    }

    #[test]
    fn test_leave_nonexistent() {
        let mut room = TrainingRoom::new("r5", builder_fundamentals(), 5);
        assert!(room.leave("ghost".to_string()).is_none());
    }

    #[test]
    fn test_record_attempt_updates_progress() {
        let mut room = TrainingRoom::new("r6", builder_fundamentals(), 5);
        room.enter("alice".to_string());
        room.record_attempt("alice", "place_blocks", 0.85);
        let alice = room.trainees.get("alice").unwrap();
        assert!((alice.progress_on("place_blocks") - 0.85).abs() < 1e-9);
        assert_eq!(alice.attempts.get("place_blocks"), Some(&1));
    }

    #[test]
    fn test_record_attempt_ignores_missing_agent() {
        let mut room = TrainingRoom::new("r7", builder_fundamentals(), 5);
        room.record_attempt("ghost", "place_blocks", 0.5);
    }

    #[test]
    fn test_record_observation_a2a_learning() {
        let mut room = TrainingRoom::new("r8", builder_fundamentals(), 5);
        room.enter("mentee".to_string());
        room.enter("mentor".to_string());
        room.record_attempt("mentor", "place_blocks", 1.0);
        room.record_observation("mentee", "mentor", "place_blocks");
        let mentee = room.trainees.get("mentee").unwrap();
        assert!((mentee.progress_on("place_blocks") - 0.30).abs() < 1e-9);
    }

    #[test]
    fn test_observation_ignores_missing_target() {
        let mut room = TrainingRoom::new("r8b", builder_fundamentals(), 5);
        room.enter("observer".to_string());
        room.record_observation("observer", "nobody", "place_blocks");
        // Should not panic, progress unchanged
        assert!((room.trainees.get("observer").unwrap().progress_on("place_blocks") - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_check_graduation_all_mastered() {
        let mut room = TrainingRoom::new("r9", builder_fundamentals(), 5);
        room.enter("alice".to_string());
        room.record_attempt("alice", "place_blocks", 1.0);
        room.record_attempt("alice", "read_blueprint", 1.0);
        room.record_attempt("alice", "conserve_materials", 1.0);
        room.record_attempt("alice", "structural_integrity", 1.0);
        assert!(room.check_graduation("alice"));
        assert_eq!(room.graduation_count, 1);
    }

    #[test]
    fn test_check_graduation_not_all_mastered() {
        let mut room = TrainingRoom::new("r10", builder_fundamentals(), 5);
        room.enter("alice".to_string());
        room.record_attempt("alice", "place_blocks", 1.0);
        assert!(!room.check_graduation("alice"));
        assert_eq!(room.graduation_count, 0);
    }

    #[test]
    fn test_check_graduation_missing_agent() {
        let mut room = TrainingRoom::new("r10b", builder_fundamentals(), 5);
        assert!(!room.check_graduation("nobody"));
        assert_eq!(room.graduation_count, 0);
    }

    #[test]
    fn test_graduated_and_active_agents() {
        let mut room = TrainingRoom::new("r11", conservation_basics(), 5);
        room.enter("alice".to_string());
        room.enter("bob".to_string());
        room.record_attempt("alice", "check_balance", 1.0);
        room.record_attempt("alice", "verify_total", 1.0);
        room.record_attempt("alice", "correct_error", 1.0);
        room.check_graduation("alice");

        let graduated = room.graduated_agents();
        let active = room.active_agents();
        assert_eq!(graduated.len(), 1);
        assert_eq!(active.len(), 1);
        assert_eq!(graduated[0].agent_id, "alice");
        assert_eq!(active[0].agent_id, "bob");
    }

    #[test]
    fn test_room_progress() {
        let mut room = TrainingRoom::new("r12", conservation_basics(), 5);
        room.enter("alice".to_string());
        room.enter("bob".to_string());
        room.record_attempt("alice", "check_balance", 1.0);
        room.record_attempt("alice", "verify_total", 0.5);
        room.record_attempt("alice", "correct_error", 0.0);
        let expected = ((1.0 + 0.5 + 0.0) / 3.0 + 0.0) / 2.0;
        assert!((room.room_progress() - expected).abs() < 1e-9);
    }

    #[test]
    fn test_room_progress_empty() {
        let room = TrainingRoom::new("r13", conservation_basics(), 5);
        assert_eq!(room.room_progress(), 0.0);
    }

    #[test]
    fn test_events() {
        let mut room = TrainingRoom::new("r14", conservation_basics(), 5);
        room.enter("alice".to_string());
        room.record_attempt("alice", "check_balance", 0.9);
        assert_eq!(room.events().len(), 2);
    }

    #[test]
    fn test_summary() {
        let mut room = TrainingRoom::new("r15", conservation_basics(), 5);
        room.enter("alice".to_string());
        room.set_mentor("mentor1".to_string());
        let s = room.summary();
        assert!(s.contains("r15"));
        assert!(s.contains("Conservation Basics"));
        assert!(s.contains("mentor1"));
    }

    #[test]
    fn test_entered_tick_tracking() {
        let mut room = TrainingRoom::new("r16", conservation_basics(), 5);
        room.tick = 42;
        room.enter("alice".to_string());
        assert_eq!(room.trainees.get("alice").unwrap().entered_tick, 42);
    }

    #[test]
    fn test_mentor_id_propagated_on_enter() {
        let mut room = TrainingRoom::new("r17", conservation_basics(), 5);
        room.set_mentor("guru".to_string());
        room.enter("alice".to_string());
        assert_eq!(
            room.trainees.get("alice").unwrap().mentor_id,
            Some("guru".to_string())
        );
    }

    // --- TrainingAcademy ---

    #[test]
    fn test_academy_new() {
        let academy = TrainingAcademy::new();
        assert!(academy.rooms.is_empty());
        assert!(academy.curricula.is_empty());
    }

    #[test]
    fn test_add_curriculum() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        assert_eq!(academy.curricula.len(), 1);
    }

    #[test]
    fn test_create_room_from_curriculum() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        let room_id = academy.create_room("conservation-basics");
        assert!(room_id.is_some());
        assert_eq!(academy.rooms.len(), 1);
    }

    #[test]
    fn test_create_room_unknown_curriculum() {
        let mut academy = TrainingAcademy::new();
        let room_id = academy.create_room("nonexistent");
        assert!(room_id.is_none());
        assert!(academy.rooms.is_empty());
    }

    #[test]
    fn test_enroll_and_track_history() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        let room_id = academy.create_room("conservation-basics").unwrap();
        assert!(academy.enroll(&room_id, "alice".to_string()));
        assert!(academy.enroll(&room_id, "bob".to_string()));
        let rooms = academy.agent_rooms("alice");
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].id.to_string(), room_id.to_string());
    }

    #[test]
    fn test_enroll_when_full() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        let mut tight = TrainingRoom::new("tight", conservation_basics(), 1);
        let tid = TrainingRoomId("tight-room".to_string());
        tight.enter("alice".to_string());
        academy.rooms.insert(tid.clone(), tight);
        academy
            .agent_history
            .entry("alice".to_string())
            .or_default()
            .push(tid.clone());
        // Second enroll should fail
        assert!(!academy.enroll(&tid, "bob".to_string()));
    }

    #[test]
    fn test_enroll_nonexistent_room() {
        let mut academy = TrainingAcademy::new();
        assert!(!academy.enroll(&TrainingRoomId("ghost".to_string()), "alice".to_string()));
    }

    #[test]
    fn test_agent_rooms_unknown() {
        let academy = TrainingAcademy::new();
        assert!(academy.agent_rooms("nobody").is_empty());
    }

    #[test]
    fn test_academy_create_multiple_rooms() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        let r1 = academy.create_room("conservation-basics").unwrap();
        let r2 = academy.create_room("conservation-basics").unwrap();
        assert_ne!(r1, r2);
        assert_eq!(academy.rooms.len(), 2);
    }

    #[test]
    fn test_academy_stats_empty() {
        let academy = TrainingAcademy::new();
        let stats = academy.academy_stats();
        assert_eq!(stats.total_rooms, 0);
        assert_eq!(stats.total_enrollments, 0);
        assert_eq!(stats.total_graduations, 0);
        assert_eq!(stats.curricula_offered, 0);
        assert_eq!(stats.active_rooms, 0);
    }

    #[test]
    fn test_academy_stats_populated() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        academy.add_curriculum(builder_fundamentals());
        let r1 = academy.create_room("conservation-basics").unwrap();
        academy.enroll(&r1, "alice".to_string());
        let stats = academy.academy_stats();
        assert_eq!(stats.total_rooms, 1);
        assert_eq!(stats.total_enrollments, 1);
        assert_eq!(stats.curricula_offered, 2);
        assert_eq!(stats.active_rooms, 1);
    }

    // --- Pre-built curricula ---

    #[test]
    fn test_conservation_basics() {
        let c = conservation_basics();
        assert_eq!(c.id, "conservation-basics");
        assert_eq!(c.skills.len(), 3);
        assert!((c.difficulty - 0.2).abs() < 1e-9);
    }

    #[test]
    fn test_builder_fundamentals() {
        let c = builder_fundamentals();
        assert_eq!(c.skills.len(), 4);
        assert!((c.difficulty - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_agent_composition_101() {
        let c = agent_composition_101();
        assert_eq!(c.skills.len(), 3);
        assert!((c.difficulty - 0.5).abs() < 1e-9);
        assert_eq!(c.prerequisites, vec!["builder-fundamentals"]);
    }

    #[test]
    fn test_advanced_farming() {
        let c = advanced_farming();
        assert_eq!(c.skills.len(), 5);
        assert!((c.difficulty - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_leadership_training() {
        let c = leadership_training();
        assert_eq!(c.skills.len(), 4);
        assert!((c.difficulty - 0.9).abs() < 1e-9);
        assert_eq!(c.prerequisites.len(), 2);
    }

    #[test]
    fn test_eval_method_name() {
        let acc = EvalMethod::ConservationAccuracy { error: 0.1, threshold: 0.2 };
        let comp = EvalMethod::TaskCompletion { time_limit_secs: 30.0 };
        assert_eq!(acc.name(), "conservation_accuracy");
        assert_eq!(comp.name(), "task_completion");
    }

    #[test]
    fn test_training_category_display() {
        assert_eq!(TrainingCategory::Leadership.to_string(), "Leadership");
        assert_eq!(TrainingCategory::Adaptation.to_string(), "Adaptation");
    }

    #[test]
    fn test_full_room_lifecycle() {
        let mut room = TrainingRoom::new("lifecycle", conservation_basics(), 5);
        room.set_mentor("professor_x".to_string());
        room.enter("student1".to_string());
        room.enter("student2".to_string());

        // student1 learns through attempts
        room.record_attempt("student1", "check_balance", 0.5);
        room.record_attempt("student1", "check_balance", 0.8);
        room.record_attempt("student1", "check_balance", 1.0);
        room.record_attempt("student1", "verify_total", 1.0);
        room.record_attempt("student1", "correct_error", 1.0);

        // student2 observes student1
        room.record_observation("student2", "student1", "check_balance");

        // student1 should graduate
        assert!(room.check_graduation("student1"));
        assert_eq!(room.graduated_agents().len(), 1);
        assert_eq!(room.active_agents().len(), 1);

        // student2 should have some progress from observation
        assert!(room.trainees.get("student2").unwrap().progress_on("check_balance") > 0.0);

        // Summary should not panic
        let summary = room.summary();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_serde_roundtrip_training_room_id() {
        let id = TrainingRoomId("hello".to_string());
        let json = serde_json::to_string(&id).unwrap();
        let back: TrainingRoomId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn test_serde_roundtrip_trainee_state() {
        let room_id = TrainingRoomId("r1".to_string());
        let mut progress = HashMap::new();
        progress.insert("skill1".to_string(), 0.8);
        let state = TraineeState {
            agent_id: "agent1".to_string(),
            skill_progress: progress,
            attempts: HashMap::new(),
            mentor_id: Some("mentor".to_string()),
            room_id: room_id.clone(),
            entered_tick: 5,
            graduated: false,
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: TraineeState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.agent_id, back.agent_id);
        assert_eq!(state.room_id, back.room_id);
        assert_eq!(state.mentor_id, back.mentor_id);
        assert!((back.progress_on("skill1") - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_serde_roundtrip_training_event() {
        let event = TrainingEvent::SkillAttempted {
            agent: "a1".to_string(),
            skill: "s1".to_string(),
            score: 0.95,
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: TrainingEvent = serde_json::from_str(&json).unwrap();
        match back {
            TrainingEvent::SkillAttempted { agent, skill, score } => {
                assert_eq!(agent, "a1");
                assert_eq!(skill, "s1");
                assert!((score - 0.95).abs() < 1e-9);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_serde_roundtrip_curriculum() {
        let c = leadership_training();
        let json = serde_json::to_string(&c).unwrap();
        let back: Curriculum = serde_json::from_str(&json).unwrap();
        assert_eq!(c.id, back.id);
        assert_eq!(c.skills.len(), back.skills.len());
        assert!((c.difficulty - back.difficulty).abs() < 1e-9);
    }

    #[test]
    fn test_serde_roundtrip_training_room() {
        let mut room = TrainingRoom::new("sroom", builder_fundamentals(), 5);
        room.set_mentor("m".to_string());
        room.enter("a1".to_string());
        room.record_attempt("a1", "place_blocks", 0.9);
        let json = serde_json::to_string(&room).unwrap();
        let back: TrainingRoom = serde_json::from_str(&json).unwrap();
        assert_eq!(room.id, back.id);
        assert_eq!(room.events().len(), back.events().len());
        assert_eq!(room.graduation_count, back.graduation_count);
    }

    #[test]
    fn test_graduation_count_tracks_multiple() {
        let mut room = TrainingRoom::new("multi", conservation_basics(), 5);
        room.enter("a".to_string());
        room.enter("b".to_string());
        room.record_attempt("a", "check_balance", 1.0);
        room.record_attempt("a", "verify_total", 1.0);
        room.record_attempt("a", "correct_error", 1.0);
        room.record_attempt("b", "check_balance", 1.0);
        room.record_attempt("b", "verify_total", 1.0);
        room.record_attempt("b", "correct_error", 1.0);
        room.check_graduation("a");
        room.check_graduation("b");
        assert_eq!(room.graduation_count, 2);
    }

    #[test]
    fn test_agent_history_multiple_rooms() {
        let mut academy = TrainingAcademy::new();
        academy.add_curriculum(conservation_basics());
        academy.add_curriculum(leadership_training());
        let r1 = academy.create_room("conservation-basics").unwrap();
        let r2 = academy.create_room("leadership-training").unwrap();
        academy.enroll(&r1, "alice".to_string());
        academy.enroll(&r2, "alice".to_string());
        assert_eq!(academy.agent_rooms("alice").len(), 2);
    }
}
