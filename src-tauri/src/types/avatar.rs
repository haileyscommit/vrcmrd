use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PerfRank {
    VeryPoor,
    Poor,
    Medium,
    Good,
    Excellent
}

impl PerfRank {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().replace("_", "").as_str() {
            "verypoor" => Some(PerfRank::VeryPoor),
            "poor" => Some(PerfRank::Poor),
            "medium" => Some(PerfRank::Medium),
            "good" => Some(PerfRank::Good),
            "excellent" => Some(PerfRank::Excellent),
            _ => None, // return None if unknown
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrcxAvatarSearchResult {
    pub id: String,
    #[serde(alias="avatarName")]
    pub name: Option<String>,
    pub author_name: Option<String>,
    pub author_id: Option<String>,
    pub image_url: Option<String>,
    // pub description: Option<String>, // completely irrelevant
    pub thumbnail_image_url: Option<String>,
    pub performance: Option<VrcxAvatarSearchPerformance>,
    pub release_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrcxAvatarSearchPerformance {
    #[serde(alias="standalonewindows")]
    pub pc_rating: Option<String>,
    #[serde(alias="android")]
    pub android_rating: Option<String>,
    #[serde(alias="ios")]
    pub ios_rating: Option<String>,
    pub has_impostor: Option<bool>,
    #[serde(alias="standalonewindows-sort")]
    pub pc_sort: Option<u8>,
    #[serde(alias="android-sort")]
    pub android_sort: Option<u8>,
    #[serde(alias="ios-sort")]
    pub ios_sort: Option<u8>,
}

pub trait GetWorstRank {
    fn get_worst_rank(&self) -> Option<PerfRank>;
    fn get_pc_rank(&self) -> Option<PerfRank>;
    fn get_android_rank(&self) -> Option<PerfRank>;
    fn get_ios_rank(&self) -> Option<PerfRank>;
}

impl GetWorstRank for VrcxAvatarSearchPerformance {
    fn get_pc_rank(&self) -> Option<PerfRank> {
        self.pc_rating.as_ref().and_then(|r| PerfRank::from_string(r))
    }

    fn get_android_rank(&self) -> Option<PerfRank> {
        self.android_rating.as_ref().and_then(|r| PerfRank::from_string(r))
    }

    fn get_ios_rank(&self) -> Option<PerfRank> {
        self.ios_rating.as_ref().and_then(|r| PerfRank::from_string(r))
    }
    
    /// Get the lowest performance rank. This is the most relevant for fairness to
    /// Quest/iOS users.
    // TODO: directly use get_pc_rank instead for PC-only worlds
    fn get_worst_rank(&self) -> Option<PerfRank> {
        let ranks = [
            self.get_pc_rank(),
            self.get_android_rank(),
            self.get_ios_rank(),
        ];
        ranks.into_iter().flatten().min_by_key(|r| match r {
            PerfRank::VeryPoor => 0,
            PerfRank::Poor => 1,
            PerfRank::Medium => 2,
            PerfRank::Good => 3,
            PerfRank::Excellent => 4,
        })
    }
}

impl GetWorstRank for vrchatapi::models::AvatarPerformance {
    fn get_pc_rank(&self) -> Option<PerfRank> {
        self.standalonewindows.as_ref().and_then(|r| PerfRank::from_string(r))
    }
    fn get_android_rank(&self) -> Option<PerfRank> {
        self.android.as_ref().and_then(|r| PerfRank::from_string(r))
    }
    fn get_ios_rank(&self) -> Option<PerfRank> {
        self.ios.as_ref().and_then(|r| PerfRank::from_string(r))
    }
    /// Get the lowest performance rank. This is the most relevant for fairness to
    /// Quest/iOS users.
    // TODO: directly use get_pc_rank instead for PC-only worlds
    fn get_worst_rank(&self) -> Option<PerfRank> {
        let ranks = [
            self.get_pc_rank(),
            self.get_android_rank(),
            self.get_ios_rank(),
        ];
        ranks.into_iter().flatten().min_by_key(|r| match r {
            PerfRank::VeryPoor => 0,
            PerfRank::Poor => 1,
            PerfRank::Medium => 2,
            PerfRank::Good => 3,
            PerfRank::Excellent => 4,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AvatarBundleFileMetadata {
    pub file_id: String,
    pub file_version: u32,
    pub avatar_name: Option<String>,
    pub author_id: String,
    pub analysis_type: AvatarBundleAnalysisType,
}
#[derive(Debug, Clone)]
pub enum AvatarBundleAnalysisType {
    Basic,
    Standard,
    Security,
}