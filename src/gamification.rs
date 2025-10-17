use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Timelike}; // ADDED: Timelike import

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudentProfile {
    pub student_name: String,
    pub total_points: i32,
    pub level: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub badges: Vec<Badge>,
    pub achievements: Vec<Achievement>,
    pub study_sessions: Vec<StudySession>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Badge {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub earned_at: DateTime<Utc>,
    pub rarity: String, // "Common", "Rare", "Epic", "Legendary"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Achievement {
    pub name: String,
    pub description: String,
    pub points: i32,
    pub progress: f64,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudySession {
    pub date: DateTime<Utc>,
    pub duration_hours: f64,
    pub subjects: Vec<String>,
    pub points_earned: i32,
    pub focus_score: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub student_name: String,
    pub total_points: i32,
    pub level: i32,
    pub badge_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GamificationResponse {
    pub profile: StudentProfile,
    pub points_earned: i32,
    pub level_up: bool,
    pub new_badges: Vec<Badge>,
    pub new_achievements: Vec<Achievement>,
    pub leaderboard_position: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StudySessionRequest {
    pub student_name: String,
    pub duration_hours: f64,
    pub subjects: Vec<String>,
    pub focus_score: f64, // 0.0 to 1.0
    pub attendance_today: bool,
}

pub struct GamificationEngine {
    achievements: HashMap<String, AchievementTemplate>,
    badges: HashMap<String, BadgeTemplate>,
}

#[derive(Clone)]
struct AchievementTemplate {
    name: String,
    description: String,
    points: i32,
    condition: AchievementCondition,
}

#[derive(Clone)]
struct BadgeTemplate {
    name: String,
    description: String,
    icon: String,
    rarity: String,
    condition: BadgeCondition,
}

#[derive(Clone)]
enum AchievementCondition {
    TotalStudyHours(f64),
    StudyStreak(i32),
    PerfectAttendance(i32),
    PointsEarned(i32),
    SessionsCompleted(i32),
    SubjectMastery(String, i32),
}

#[derive(Clone)]
enum BadgeCondition {
    FirstStudySession,
    WeeklyChampion,
    MarathonStudier(f64),
    PerfectWeek,
    SubjectExpert(String),
    EarlyBird,
    NightOwl,
}

impl GamificationEngine {
    pub fn new() -> Self {
        let mut achievements = HashMap::new();
        let mut badges = HashMap::new();

        // Define achievements
        achievements.insert("first_steps".to_string(), AchievementTemplate {
            name: "First Steps".to_string(),
            description: "Complete your first study session".to_string(),
            points: 50,
            condition: AchievementCondition::SessionsCompleted(1),
        });

        achievements.insert("dedicated_learner".to_string(), AchievementTemplate {
            name: "Dedicated Learner".to_string(),
            description: "Reach 50 total study hours".to_string(),
            points: 200,
            condition: AchievementCondition::TotalStudyHours(50.0),
        });

        achievements.insert("study_streak_7".to_string(), AchievementTemplate {
            name: "Weekly Warrior".to_string(),
            description: "Maintain a 7-day study streak".to_string(),
            points: 150,
            condition: AchievementCondition::StudyStreak(7),
        });

        achievements.insert("perfect_month".to_string(), AchievementTemplate {
            name: "Perfect Month".to_string(),
            description: "Perfect attendance for 30 days".to_string(),
            points: 500,
            condition: AchievementCondition::PerfectAttendance(30),
        });

        achievements.insert("point_master".to_string(), AchievementTemplate {
            name: "Point Master".to_string(),
            description: "Earn 1000 total points".to_string(),
            points: 300,
            condition: AchievementCondition::PointsEarned(1000),
        });

        // Define badges
        badges.insert("first_session".to_string(), BadgeTemplate {
            name: "First Session".to_string(),
            description: "Completed your first study session".to_string(),
            icon: "🎯".to_string(),
            rarity: "Common".to_string(),
            condition: BadgeCondition::FirstStudySession,
        });

        badges.insert("weekly_champion".to_string(), BadgeTemplate {
            name: "Weekly Champion".to_string(),
            description: "Top performer for the week".to_string(),
            icon: "🏆".to_string(),
            rarity: "Rare".to_string(),
            condition: BadgeCondition::WeeklyChampion,
        });

        badges.insert("marathon_studier".to_string(), BadgeTemplate {
            name: "Marathon Studier".to_string(),
            description: "Study for 5+ hours in one session".to_string(),
            icon: "🏃‍♂️".to_string(),
            rarity: "Epic".to_string(),
            condition: BadgeCondition::MarathonStudier(5.0),
        });

        badges.insert("perfect_week".to_string(), BadgeTemplate {
            name: "Perfect Week".to_string(),
            description: "Perfect attendance and study goals for a week".to_string(),
            icon: "⭐".to_string(),
            rarity: "Rare".to_string(),
            condition: BadgeCondition::PerfectWeek,
        });

        badges.insert("early_bird".to_string(), BadgeTemplate {
            name: "Early Bird".to_string(),
            description: "Complete 10 morning study sessions".to_string(),
            icon: "🌅".to_string(),
            rarity: "Common".to_string(),
            condition: BadgeCondition::EarlyBird,
        });

        badges.insert("night_owl".to_string(), BadgeTemplate {
            name: "Night Owl".to_string(),
            description: "Complete 10 evening study sessions".to_string(),
            icon: "🌙".to_string(),
            rarity: "Common".to_string(),
            condition: BadgeCondition::NightOwl,
        });

        Self { achievements, badges }
    }

    pub fn calculate_points(&self, session: &StudySessionRequest) -> i32 {
        let mut points = (session.duration_hours * 10.0) as i32; // Base points
        points += (session.focus_score * 20.0) as i32; // Focus bonus
        points += session.subjects.len() as i32 * 5; // Multi-subject bonus
        
        if session.attendance_today {
            points += 25; // Attendance bonus
        }

        // Streak multiplier (simulated)
        if session.duration_hours > 2.0 {
            points += 15; // Long session bonus
        }

        points.max(0)
    }

    pub fn calculate_level(&self, total_points: i32) -> i32 {
        (total_points as f64 / 100.0).sqrt() as i32 + 1
    }

    pub fn check_achievements(&self, profile: &StudentProfile, _session: &StudySessionRequest) -> Vec<Achievement> {
        let mut new_achievements = Vec::new();
        let total_study_hours: f64 = profile.study_sessions.iter().map(|s| s.duration_hours).sum();
        let total_sessions = profile.study_sessions.len() as i32;

        for (_, template) in &self.achievements {
            if !profile.achievements.iter().any(|a| a.name == template.name) {
                let progress = match &template.condition {
                    AchievementCondition::TotalStudyHours(target) => {
                        total_study_hours / target
                    }
                    AchievementCondition::StudyStreak(target) => {
                        profile.current_streak as f64 / *target as f64
                    }
                    AchievementCondition::PerfectAttendance(target) => {
                        // Simplified - would need actual attendance data
                        profile.current_streak as f64 / *target as f64
                    }
                    AchievementCondition::PointsEarned(target) => {
                        profile.total_points as f64 / *target as f64
                    }
                    AchievementCondition::SessionsCompleted(target) => {
                        total_sessions as f64 / *target as f64
                    }
                    AchievementCondition::SubjectMastery(_, _) => 0.0, // Simplified
                };

                if progress >= 1.0 {
                    new_achievements.push(Achievement {
                        name: template.name.clone(),
                        description: template.description.clone(),
                        points: template.points,
                        progress: 1.0,
                        completed: true,
                        completed_at: Some(Utc::now()),
                    });
                }
            }
        }

        new_achievements
    }

    pub fn check_badges(&self, profile: &StudentProfile, session: &StudySessionRequest) -> Vec<Badge> {
        let mut new_badges = Vec::new();
        let total_sessions = profile.study_sessions.len();

        for (_, template) in &self.badges {
            if !profile.badges.iter().any(|b| b.name == template.name) {
                let earned = match &template.condition {
                    BadgeCondition::FirstStudySession => total_sessions == 1,
                    BadgeCondition::WeeklyChampion => {
                        // Simplified - would need leaderboard data
                        profile.total_points > 500
                    }
                    BadgeCondition::MarathonStudier(hours) => session.duration_hours >= *hours,
                    BadgeCondition::PerfectWeek => {
                        profile.current_streak >= 7 && session.attendance_today
                    }
                    BadgeCondition::SubjectExpert(_) => {
                        // Simplified - would need subject tracking
                        session.subjects.len() >= 3
                    }
                    BadgeCondition::EarlyBird => {
                        // FIXED: Now using Timelike trait
                        Utc::now().hour() < 12
                    }
                    BadgeCondition::NightOwl => {
                        // FIXED: Now using Timelike trait
                        Utc::now().hour() >= 18
                    }
                };

                if earned {
                    new_badges.push(Badge {
                        name: template.name.clone(),
                        description: template.description.clone(),
                        icon: template.icon.clone(),
                        earned_at: Utc::now(),
                        rarity: template.rarity.clone(),
                    });
                }
            }
        }

        new_badges
    }

    pub fn update_streak(&self, profile: &StudentProfile, _session: &StudySessionRequest) -> i32 {
        let today = Utc::now().date_naive();
        let last_activity = profile.last_activity.date_naive();
        
        if today == last_activity {
            profile.current_streak
        } else if today == last_activity.succ_opt().unwrap_or(last_activity) {
            profile.current_streak + 1
        } else {
            1
        }
    }
}

// Mock data for demonstration
pub fn get_mock_leaderboard() -> Vec<LeaderboardEntry> {
    vec![
        LeaderboardEntry {
            rank: 1,
            student_name: "Denis Lemayian".to_string(),
            total_points: 1250,
            level: 12,
            badge_count: 8,
        },
        LeaderboardEntry {
            rank: 2,
            student_name: "Saitoti Smith".to_string(),
            total_points: 980,
            level: 10,
            badge_count: 6,
        },
        LeaderboardEntry {
            rank: 3,
            student_name: "Kukutia Johnson".to_string(),
            total_points: 750,
            level: 9,
            badge_count: 4,
        },
        LeaderboardEntry {
            rank: 4,
            student_name: "Kirionki Williams".to_string(),
            total_points: 620,
            level: 8,
            badge_count: 3,
        },
        LeaderboardEntry {
            rank: 5,
            student_name: "David Lemoita".to_string(),
            total_points: 540,
            level: 7,
            badge_count: 5,
        },
    ]
}

pub fn get_mock_profile(student_name: &str) -> StudentProfile {
    StudentProfile {
        student_name: student_name.to_string(),
        total_points: 750,
        level: 9,
        current_streak: 5,
        longest_streak: 12,
        badges: vec![
            Badge {
                name: "First Session".to_string(),
                description: "Completed your first study session".to_string(),
                icon: "🎯".to_string(),
                earned_at: Utc::now(),
                rarity: "Common".to_string(),
            },
            Badge {
                name: "Weekly Champion".to_string(),
                description: "Top performer for the week".to_string(),
                icon: "🏆".to_string(),
                earned_at: Utc::now(),
                rarity: "Rare".to_string(),
            },
        ],
        achievements: vec![
            Achievement {
                name: "First Steps".to_string(),
                description: "Complete your first study session".to_string(),
                points: 50,
                progress: 1.0,
                completed: true,
                completed_at: Some(Utc::now()),
            },
            Achievement {
                name: "Dedicated Learner".to_string(),
                description: "Reach 50 total study hours".to_string(),
                points: 200,
                progress: 0.8,
                completed: false,
                completed_at: None,
            },
        ],
        study_sessions: vec![
            StudySession {
                date: Utc::now(),
                duration_hours: 2.5,
                subjects: vec!["Mathematics".to_string(), "Programming".to_string()],
                points_earned: 45,
                focus_score: 0.8,
            },
        ],
        last_activity: Utc::now(),
    }
}