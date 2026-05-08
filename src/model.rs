use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code, non_snake_case)]
#[serde(default)]
pub struct SaveFile {
    pub unity: UnityInfo,
    pub campaign: CampaignSave,
    #[serde(alias = "campaign_meta")]
    pub campaignMeta: Option<CampaignSaveMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code, non_snake_case)]
#[serde(default)]
pub struct UnityInfo {
    #[serde(alias = "metadata_size")]
    pub metadataSize: u32,
    #[serde(alias = "file_size")]
    pub fileSize: u32,
    pub version: u32,
    #[serde(alias = "data_offset")]
    pub dataOffset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct CampaignSave {
    // 核心进度
    #[serde(alias = "serialized_version")]
    pub serializedVersion: i32,
    pub seed: i32,
    #[serde(alias = "level_states")]
    pub levelStates: Vec<LevelState>,
    pub heroes: Vec<HeroDefinition>,
    pub inventory: Vec<SerializableHeroUpgrade>,
    #[serde(alias = "viking_frontier_position")]
    pub vikingFrontierPosition: i32,
    #[serde(alias = "coin_bank")]
    pub coinBank: i32,
    #[serde(alias = "has_checkpoint")]
    pub hasCheckpoint: bool,
    #[serde(alias = "deploy_order")]
    pub deployOrder: Vec<i32>,
    #[serde(alias = "old_deploy_order")]
    pub oldDeployOrder: Vec<i32>,

    // 游戏状态
    #[serde(alias = "last_played_level_idx")]
    pub lastPlayedLevelIdx: i32,
    #[serde(alias = "turn_count")]
    pub turnCount: i32,
    #[serde(alias = "battle_count")]
    pub battleCount: i32,
    #[serde(alias = "battles_won")]
    pub battlesWon: i32,
    #[serde(alias = "has_any_checkpoints")]
    pub hasAnyCheckpoints: bool,
    #[serde(alias = "saved_play_time")]
    pub savedPlayTime: i32,
    #[serde(alias = "save_timestamp", alias = "saveTimeStamp")]
    pub saveTimestamp: i64,
    #[serde(alias = "vikings_seen")]
    pub vikingsSeen: Vec<SerializeFriendlyIntEnum>,
    #[serde(alias = "turn_start_day")]
    pub turnStartDay: f32,
    pub day: f32,
    #[serde(alias = "game_over_reason")]
    pub gameOverReason: i32,
    pub stats: CampaignStats,
    pub prefs: CampaignPrefs,
    #[serde(alias = "perfect_defence_streak")]
    pub perfectDefenceStreak: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct CampaignSaveMeta {
    pub metaVersion: i32,
    #[serde(alias = "serilizedVersion")]
    pub serializedVersion: i32,
    pub seed: i32,
    pub savedTime: i64,
    pub playTime: i32,
    pub campaignFraction: f32,
    pub checkpointPlayTime: i32,
    pub checkpointFraction: f32,
    pub gameOverReason: i32,
    pub prefs: CampaignPrefs,
    pub hasCheckpoint: bool,
    pub checkpointReloads: i32,
    pub displayName: String,
    pub campaignNumber: i32,
    pub saveSlot: i32,
    pub targetFileName: String,
    pub metaFileName: String,
    pub checkpointFileName: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct LevelState {
    // 岛屿基础信息
    #[serde(alias = "name_term")]
    pub nameTerm: String,
    pub seed: i32,
    #[serde(alias = "frontier_depth")]
    pub frontierDepth: u8,
    #[serde(alias = "steps_from_start")]
    pub stepsFromStart: u8,
    #[serde(alias = "steps_from_end")]
    pub stepsFromEnd: u8,
    pub index: u8,
    pub width: u8,
    pub height: u8,
    pub _unlocked: bool,
    pub houses: Vec<HouseState>,
    pub _item: Option<SerializableHeroUpgrade>,
    #[serde(alias = "hero_id")]
    pub heroId: i32,
    #[serde(alias = "checkpoint_state")]
    pub checkpointState: u8,

    // 波次配置
    #[serde(alias = "play_count")]
    pub playCount: u8,
    #[serde(alias = "coin_count")]
    pub coinCount: u8,
    #[serde(alias = "coin_target")]
    pub coinTarget: u8,
    #[serde(alias = "bounty_per_wave")]
    pub bountyPerWave: i16,
    #[serde(alias = "waves_count")]
    pub wavesCount: u8,
    #[serde(alias = "min_wave_spacing")]
    pub minWaveSpacing: u8,
    #[serde(alias = "max_wave_spacing")]
    pub maxWaveSpacing: u8,
    #[serde(alias = "relative_difficulty")]
    pub relativeDifficulty: u8,
    #[serde(alias = "good_seed")]
    pub goodSeed: bool,
    #[serde(alias = "meta_reward")]
    pub metaReward: bool,
    /// Islands reachable from this one – determines navigation options.
    pub connections: Vec<i32>,
    #[serde(alias = "turn_count")]
    pub turnCount: Option<i32>,
    #[serde(alias = "battle_count")]
    pub battleCount: Option<i32>,
    #[serde(alias = "deploy_order")]
    pub deployOrder: Option<Vec<i32>>,
    #[serde(alias = "old_deploy_order")]
    pub oldDeployOrder: Option<Vec<i32>>,

    // UI-only（不持久化）
    #[serde(skip)]
    pub posX: f32,
    #[serde(skip)]
    pub posY: f32,
    #[serde(skip)]
    pub rectMinX: i32,
    #[serde(skip)]
    pub rectMinY: i32,
    #[serde(skip)]
    pub rectWidth: i32,
    #[serde(skip)]
    pub rectHeight: i32,
    #[serde(skip)]
    pub hasSprite: bool,

    // 运行时引用
    #[serde(skip)]
    pub objectReferences: Vec<LevelObjectReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct HouseState {
    // 房屋状态
    pub condition: u8,
    pub value: u8,

    // 渲染属性（不持久化）
    #[serde(skip)]
    pub xMin: f32,
    #[serde(skip)]
    pub yMin: f32,
    #[serde(skip)]
    pub width: f32,
    #[serde(skip)]
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct LevelObjectReference {
    pub name: Option<String>,
    pub key: Option<LevelObjectReferenceKey>,
    pub id: Option<i32>,
    #[serde(alias = "path_id")]
    pub pathId: Option<i32>,
    #[serde(alias = "type_id")]
    pub typeId: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct LevelObjectReferenceKey {
    pub value: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct HeroDefinition {
    // 英雄基础信息
    pub id: i32,
    #[serde(alias = "name_term")]
    pub nameTerm: String,
    pub hue: f32,
    #[serde(alias = "voice_name")]
    pub voiceName: String,
    #[serde(alias = "death_level_id")]
    pub deathLevelId: i32,
    pub recruited: bool,
    #[serde(alias = "_alive")]
    pub alive: bool,
    #[serde(alias = "_coins")]
    pub coins: i32,
    #[serde(alias = "has_crown")]
    pub hasCrown: bool,
    #[serde(alias = "crown_style")]
    pub crownStyle: Option<String>,
    #[serde(alias = "propertyBank", alias = "property_bank", alias = "<propertyBank>k__BackingField")]
    pub propertyBank: Option<i32>,
    #[serde(alias = "_color")]
    pub color: Option<i32>,

    // 升级与回合状态
    #[serde(alias = "class_upgrade")]
    pub classUpgrade: Option<SerializableHeroUpgrade>,
    #[serde(alias = "item_upgrade")]
    pub itemUpgrade: Option<SerializableHeroUpgrade>,
    #[serde(alias = "skill_upgrade")]
    pub skillUpgrade: Option<SerializableHeroUpgrade>,
    #[serde(alias = "trait_upgrade")]
    pub traitUpgrade: Option<SerializableHeroUpgrade>,
    #[serde(alias = "max_uses_per_turn")]
    pub maxUsesPerTurn: u8,
    #[serde(alias = "times_used_this_turn")]
    pub timesUsedThisTurn: u8,
    pub discount: f32,
    #[serde(alias = "discount_type")]
    pub discountType: i32,
    pub statistics: HeroStats,
    pub available: bool,
    #[serde(alias = "available_this_turn")]
    pub availableThisTurn: bool,

    // 队伍管理
    #[serde(alias = "max_soldiers", alias = "<maxSoldiers>k__BackingField")]
    pub maxSoldiers: i32,
    #[serde(alias = "squad_level", alias = "<squadLevel>k__BackingField")]
    pub squadLevel: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct HeroStats {
    #[serde(alias = "recruited_turn", alias = "<recruitedTurn>k__BackingField")]
    pub recruitedTurn: i32,
    #[serde(alias = "vikings_killed", alias = "<Voxels.TowerDefense.IHeroStats.vikingsKilled>k__BackingField")]
    pub vikingsKilled: i32,
    #[serde(alias = "islands_visited", alias = "<Voxels.TowerDefense.IHeroStats.islandsVisited>k__BackingField")]
    pub islandsVisited: i32,
    #[serde(alias = "islands_won", alias = "<Voxels.TowerDefense.IHeroStats.islandsWon>k__BackingField")]
    pub islandsWon: i32,
    #[serde(alias = "soldiers_lost", alias = "<Voxels.TowerDefense.IHeroStats.soldiersLost>k__BackingField")]
    pub soldiersLost: i32,
}

// 升级编码
#[allow(dead_code)]
pub const WAR_HORN_UPGRADE_CODE: &str = "Hero_Upgrade_Horn";
#[allow(dead_code)]
pub const GRAIL_UPGRADE_CODE: &str = "Hero_Upgrade_Grail";
#[allow(dead_code)]
pub const BOMB_UPGRADE_CODE: &str = "Hero_Upgrade_Bomb";
#[allow(dead_code)]
pub const MINE_UPGRADE_CODE: &str = "Hero_Upgrade_Mine";
#[allow(dead_code)]
pub const PHILOSOPHERS_STONE_UPGRADE_CODE: &str = "Hero_Upgrade_PhilosophersStone";
#[allow(dead_code)]
pub const SIZE_UPGRADE_CODE: &str = "Hero_Upgrade_Size";
#[allow(dead_code)]
pub const WARHAMMER_UPGRADE_CODE: &str = "Hero_Upgrade_Warhammer";
#[allow(dead_code)]
pub const CORNUCOPIA_UPGRADE_CODE: &str = "Hero_Upgrade_Cornucopia";

// 特性编码：掷斧手、迅捷精通、追猎、荆棘
#[allow(dead_code)]
pub const TRAIT_AXE_THROWER_CODE: &str = "Hero_Trait_AxeThrower";
#[allow(dead_code)]
pub const TRAIT_CHEAPER_CLASS_CODE: &str = "Hero_Trait_CheaperClass";
#[allow(dead_code)]
pub const TRAIT_REGENERATIVE_CODE: &str = "Hero_Trait_Regenerative";
#[allow(dead_code)]
pub const TRAIT_THORN_CODE: &str = "Hero_Trait_Thorn";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct SerializableHeroUpgrade {
    pub name: String,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct CampaignStats {
    pub version: i32,
    #[serde(alias = "vikings_killed")]
    pub vikingsKilled: i32,
    #[serde(alias = "english_killed")]
    pub englishKilled: i32,
    #[serde(alias = "heroes_recruited")]
    pub heroesRecruited: i32,
    #[serde(alias = "heroes_died")]
    pub heroesDied: i32,
    #[serde(alias = "islands_visited")]
    pub islandsVisited: i32,
    #[serde(alias = "islands_defended")]
    pub islandsDefended: i32,
    #[serde(alias = "islands_lost")]
    pub islandsLost: i32,
    #[serde(alias = "islands_fled")]
    pub islandsFled: i32,
    #[serde(alias = "unique_islands_visited")]
    pub uniqueIslandsVisited: i32,
    #[serde(alias = "unique_islands_defended")]
    pub uniqueIslandsDefended: i32,
    #[serde(alias = "unique_islands_lost")]
    pub uniqueIslandsLost: i32,
    #[serde(alias = "coins_collected")]
    pub coinsCollected: i32,
    #[serde(alias = "checkpoints_saved")]
    pub checkpointsSaved: i32,
    #[serde(alias = "checkpoints_lost")]
    pub checkpointsLost: i32,
    #[serde(alias = "level_restarts")]
    pub levelRestarts: i32,
    #[serde(alias = "viking_types_killed")]
    pub vikingTypesKilled: Vec<SerializeFriendlyIntEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct CampaignPrefs {
    pub difficulty: i32,
    #[serde(alias = "skip_tutorial")]
    pub skipTutorial: bool,
    #[serde(alias = "allow_replays")]
    pub allowReplays: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SerializeFriendlyIntEnum {
    Wrapped(SerializeFriendlyEnum<i32>),
    Value(i32),
}

impl Default for SerializeFriendlyIntEnum {
    fn default() -> Self {
        Self::Value(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct SerializeFriendlyEnum<T> {
    #[serde(alias = "_value")]
    pub value: T,
    #[serde(alias = "_valueString")]
    pub valueString: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_save_decodes_old_snake_case_json() {
        let json = r#"{
          "serialized_version": 19,
          "seed": 11,
          "level_states": [],
          "heroes": [],
          "inventory": [],
          "last_played_level_idx": 2,
          "viking_frontier_position": 3,
          "turn_count": 4,
          "battle_count": 5,
          "battles_won": 6,
          "coin_bank": 123,
          "has_any_checkpoints": true,
          "saved_play_time": 77,
          "save_timestamp": 888,
          "deploy_order": [1,2],
          "old_deploy_order": [2,1],
          "vikings_seen": [1, {"value":2,"valueString":"Raider"}],
          "turn_start_day": 1.5,
          "day": 2.5,
          "game_over_reason": 0,
          "stats": {"version":1},
          "prefs": {"difficulty":2, "skip_tutorial":true, "allow_replays":false},
          "has_checkpoint": false,
          "perfect_defence_streak": 1
        }"#;
        let model: CampaignSave = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(model.serializedVersion, 19);
        assert_eq!(model.coinBank, 123);
        assert_eq!(model.turnCount, 4);
        assert!(model.prefs.skipTutorial);
        assert_eq!(model.vikingsSeen.len(), 2);
    }

    #[test]
    fn campaign_save_encodes_csharp_style_field_names() {
        let save = CampaignSave {
            serializedVersion: 19,
            coinBank: 42,
            prefs: CampaignPrefs {
                difficulty: 3,
                skipTutorial: true,
                allowReplays: false,
            },
            ..Default::default()
        };
        let value = serde_json::to_value(save).expect("encode should succeed");
        let obj = value.as_object().expect("campaign save json object");
        assert!(obj.contains_key("serializedVersion"));
        assert!(obj.contains_key("coinBank"));
        assert!(obj.get("levelAtlas").is_none());
        assert!(obj.get("paintAtlas").is_none());
    }

    #[test]
    fn campaign_save_meta_stays_lightweight() {
        let meta = CampaignSaveMeta::default();
        let value = serde_json::to_value(meta).expect("encode should succeed");
        let obj = value.as_object().expect("campaign save meta json object");
        assert!(obj.contains_key("metaVersion"));
        assert!(obj.get("weatherSystem").is_none());
        assert!(obj.get("levelAtlas").is_none());
    }

    #[test]
    fn level_state_omits_p2_display_fields() {
        let state = LevelState {
            nameTerm: "island_01".to_string(),
            seed: 42,
            index: 3,
            _unlocked: true,
            posX: 1.5,
            posY: 2.5,
            rectMinX: 10,
            rectMinY: 20,
            rectWidth: 100,
            rectHeight: 80,
            hasSprite: true,
            objectReferences: vec![LevelObjectReference {
                name: Some("obj".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let value = serde_json::to_value(state).expect("encode should succeed");
        let obj = value.as_object().expect("level state json object");

        // P0/P1 fields must be present
        assert!(obj.contains_key("nameTerm"), "nameTerm should be serialized (P0)");
        assert!(obj.contains_key("seed"), "seed should be serialized (P0)");
        assert!(obj.contains_key("index"), "index should be serialized (P0)");
        assert!(obj.contains_key("_unlocked"), "unlocked should be serialized (P0)");
        assert!(obj.contains_key("connections"), "connections should be serialized (P1)");

        // P2/P3 fields must be absent
        assert!(obj.get("posX").is_none(), "posX must be excluded (P2)");
        assert!(obj.get("posY").is_none(), "posY must be excluded (P2)");
        assert!(obj.get("rectMinX").is_none(), "rectMinX must be excluded (P2)");
        assert!(obj.get("rectMinY").is_none(), "rectMinY must be excluded (P2)");
        assert!(obj.get("rectWidth").is_none(), "rectWidth must be excluded (P2)");
        assert!(obj.get("rectHeight").is_none(), "rectHeight must be excluded (P2)");
        assert!(obj.get("hasSprite").is_none(), "hasSprite must be excluded (P2)");
        assert!(obj.get("objectReferences").is_none(), "objectReferences must be excluded (P3)");
    }

    #[test]
    fn house_state_omits_p2_layout_fields() {
        let house = HouseState {
            condition: 2,
            value: 5,
            xMin: 10.0,
            yMin: 20.0,
            width: 50.0,
            height: 30.0,
        };
        let value = serde_json::to_value(house).expect("encode should succeed");
        let obj = value.as_object().expect("house state json object");

        // P0 fields must be present
        assert!(obj.contains_key("condition"), "condition should be serialized (P0)");
        assert!(obj.contains_key("value"), "value should be serialized (P0)");

        // P2/P3 layout fields must be absent
        assert!(obj.get("xMin").is_none(), "xMin must be excluded (P2)");
        assert!(obj.get("yMin").is_none(), "yMin must be excluded (P2)");
        assert!(obj.get("width").is_none(), "width must be excluded (P2)");
        assert!(obj.get("height").is_none(), "height must be excluded (P2)");
    }

    #[test]
    fn level_state_round_trips_p0_p1_fields() {
        let json = r#"{
          "nameTerm": "test_island",
          "seed": 99,
          "index": 1,
          "_unlocked": true,
          "play_count": 3,
          "coin_count": 2,
          "coin_target": 5,
          "connections": [2, 3],
          "turn_count": 7,
          "battle_count": 4,
          "posX": 1.5,
          "posY": 2.5,
          "rectMinX": 10,
          "hasSprite": true,
          "objectReferences": [{"name": "obj"}]
        }"#;
        let state: LevelState = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(state.nameTerm, "test_island");
        assert_eq!(state.seed, 99);
        assert_eq!(state.connections, vec![2, 3]);
        assert_eq!(state.turnCount, Some(7));

        // P2/P3 fields are stripped even when present in the input
        let out = serde_json::to_value(&state).expect("encode should succeed");
        let obj = out.as_object().unwrap();
        assert!(obj.get("posX").is_none());
        assert!(obj.get("hasSprite").is_none());
        assert!(obj.get("objectReferences").is_none());
    }

    #[test]
    fn hero_definition_supports_legacy_alive_and_coins_names() {
        let json = r#"{
          "id": 1,
          "nameTerm": "hero_name",
          "_alive": true,
          "_coins": 50
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert!(hero.alive);
        assert_eq!(hero.coins, 50);
    }

    #[test]
    fn hero_definition_supports_max_soldiers_and_squad_level() {
        let json = r#"{
          "id": 2,
          "nameTerm": "hero_name",
          "maxSoldiers": 8,
          "squadLevel": 3
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.maxSoldiers, 8);
        assert_eq!(hero.squadLevel, 3);
    }

    #[test]
    fn hero_definition_supports_snake_case_max_soldiers_and_squad_level() {
        let json = r#"{
          "id": 3,
          "nameTerm": "hero_name",
          "max_soldiers": 6,
          "squad_level": 2
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.maxSoldiers, 6);
        assert_eq!(hero.squadLevel, 2);
    }

    #[test]
    fn hero_definition_supports_backing_field_names_for_max_soldiers_and_squad_level() {
        // Verify that the C# auto-property backing field names are accepted as aliases.
        let json = r#"{
          "id": 4,
          "nameTerm": "hero_name",
          "<maxSoldiers>k__BackingField": 8,
          "<squadLevel>k__BackingField": 1
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.maxSoldiers, 8);
        assert_eq!(hero.squadLevel, 1);
    }

    #[test]
    fn hero_stats_serializes_soldiers_lost() {
        let stats = HeroStats {
            recruitedTurn: 1,
            vikingsKilled: 10,
            islandsVisited: 3,
            islandsWon: 2,
            soldiersLost: 5,
        };
        let value = serde_json::to_value(&stats).expect("encode should succeed");
        let obj = value.as_object().expect("HeroStats json object");
        assert!(obj.contains_key("soldiersLost"), "soldiersLost should be serialized");
        assert_eq!(obj["soldiersLost"], 5);
    }

    #[test]
    fn hero_stats_deserializes_soldiers_lost_camel_case() {
        let json = r#"{
          "recruitedTurn": 1,
          "vikingsKilled": 10,
          "islandsVisited": 3,
          "islandsWon": 2,
          "soldiersLost": 5
        }"#;
        let stats: HeroStats = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(stats.soldiersLost, 5);
    }

    #[test]
    fn hero_stats_deserializes_soldiers_lost_snake_case() {
        let json = r#"{"soldiers_lost": 7}"#;
        let stats: HeroStats = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(stats.soldiersLost, 7);
    }

    #[test]
    fn hero_stats_deserializes_soldiers_lost_explicit_interface_backing_field() {
        // C# explicit interface implementation:
        // <Voxels.TowerDefense.IHeroStats.soldiersLost>k__BackingField
        let json = r#"{"<Voxels.TowerDefense.IHeroStats.soldiersLost>k__BackingField": 3}"#;
        let stats: HeroStats = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(stats.soldiersLost, 3);
    }

    #[test]
    fn hero_definition_round_trips_max_soldiers_and_statistics() {
        let hero = HeroDefinition {
            id: 1,
            nameTerm: "hero_name".to_string(),
            maxSoldiers: 8,
            statistics: HeroStats {
                soldiersLost: 3,
                ..Default::default()
            },
            ..Default::default()
        };
        let json = serde_json::to_value(&hero).expect("encode should succeed");
        let obj = json.as_object().expect("HeroDefinition json object");
        assert!(obj.contains_key("maxSoldiers"), "maxSoldiers must be present in JSON");
        assert_eq!(obj["maxSoldiers"], 8);

        let stats_obj = obj["statistics"].as_object().expect("statistics json object");
        assert!(stats_obj.contains_key("soldiersLost"), "soldiersLost must be present in statistics");
        assert_eq!(stats_obj["soldiersLost"], 3);
    }

    #[test]
    fn hero_definition_decodes_property_bank_camel_case() {
        let json = r#"{
          "id": 5,
          "nameTerm": "hero_name",
          "propertyBank": 42
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.propertyBank, Some(42));
    }

    #[test]
    fn hero_definition_decodes_property_bank_snake_case() {
        let json = r#"{
          "id": 6,
          "nameTerm": "hero_name",
          "property_bank": 7
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.propertyBank, Some(7));
    }

    #[test]
    fn hero_definition_decodes_property_bank_backing_field() {
        let json = r#"{
          "id": 7,
          "nameTerm": "hero_name",
          "<propertyBank>k__BackingField": 99
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.propertyBank, Some(99));
    }

    #[test]
    fn hero_definition_property_bank_defaults_to_none() {
        let json = r#"{"id": 8, "nameTerm": "hero_name"}"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.propertyBank, None);
    }

    #[test]
    fn hero_definition_decodes_color_via_underscore_alias() {
        let json = r#"{
          "id": 9,
          "nameTerm": "hero_name",
          "_color": 15
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.color, Some(15));
    }

    #[test]
    fn hero_definition_color_defaults_to_none() {
        let json = r#"{"id": 10, "nameTerm": "hero_name"}"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        assert_eq!(hero.color, None);
    }

    #[test]
    fn hero_definition_round_trips_property_bank_and_color() {
        let hero = HeroDefinition {
            id: 11,
            nameTerm: "hero_name".to_string(),
            propertyBank: Some(42),
            color: Some(15),
            ..Default::default()
        };
        let json = serde_json::to_value(&hero).expect("encode should succeed");
        let obj = json.as_object().expect("HeroDefinition json object");
        assert!(obj.contains_key("propertyBank"), "propertyBank must be present in JSON");
        assert_eq!(obj["propertyBank"], 42);
        assert!(obj.contains_key("color"), "color must be present in JSON");
        assert_eq!(obj["color"], 15);
    }

    #[test]
    fn upgrade_code_constants_match_game_data() {
        assert_eq!(WAR_HORN_UPGRADE_CODE, "Hero_Upgrade_Horn");
        assert_eq!(GRAIL_UPGRADE_CODE, "Hero_Upgrade_Grail");
        assert_eq!(BOMB_UPGRADE_CODE, "Hero_Upgrade_Bomb");
        assert_eq!(MINE_UPGRADE_CODE, "Hero_Upgrade_Mine");
        assert_eq!(PHILOSOPHERS_STONE_UPGRADE_CODE, "Hero_Upgrade_PhilosophersStone");
        assert_eq!(SIZE_UPGRADE_CODE, "Hero_Upgrade_Size");
        assert_eq!(WARHAMMER_UPGRADE_CODE, "Hero_Upgrade_Warhammer");
        assert_eq!(CORNUCOPIA_UPGRADE_CODE, "Hero_Upgrade_Cornucopia");
    }

    #[test]
    fn trait_code_constants_match_modifier_definitions() {
        // 掷斧手 / 迅捷精通 / 追猎 / 荆棘
        assert_eq!(TRAIT_AXE_THROWER_CODE, "Hero_Trait_AxeThrower");
        assert_eq!(TRAIT_CHEAPER_CLASS_CODE, "Hero_Trait_CheaperClass");
        assert_eq!(TRAIT_REGENERATIVE_CODE, "Hero_Trait_Regenerative");
        assert_eq!(TRAIT_THORN_CODE, "Hero_Trait_Thorn");
    }

    #[test]
    fn hero_definition_parses_trait_upgrade_axe_thrower() {
        let json = r#"{
          "id": 20,
          "nameTerm": "hero_name",
          "traitUpgrade": { "name": "Hero_Trait_AxeThrower", "level": 1 }
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        let trait_up = hero.traitUpgrade.as_ref().expect("traitUpgrade should be present");
        assert_eq!(trait_up.name, TRAIT_AXE_THROWER_CODE);
        assert_eq!(trait_up.level, 1);
    }

    #[test]
    fn hero_definition_parses_trait_upgrade_cheaper_class() {
        let json = r#"{
          "id": 21,
          "nameTerm": "hero_name",
          "traitUpgrade": { "name": "Hero_Trait_CheaperClass", "level": 2 }
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        let trait_up = hero.traitUpgrade.as_ref().expect("traitUpgrade should be present");
        assert_eq!(trait_up.name, TRAIT_CHEAPER_CLASS_CODE);
        assert_eq!(trait_up.level, 2);
    }

    #[test]
    fn hero_definition_parses_trait_upgrade_regenerative() {
        let json = r#"{
          "id": 22,
          "nameTerm": "hero_name",
          "traitUpgrade": { "name": "Hero_Trait_Regenerative", "level": 1 }
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        let trait_up = hero.traitUpgrade.as_ref().expect("traitUpgrade should be present");
        assert_eq!(trait_up.name, TRAIT_REGENERATIVE_CODE);
        assert_eq!(trait_up.level, 1);
    }

    #[test]
    fn hero_definition_parses_trait_upgrade_thorn() {
        let json = r#"{
          "id": 23,
          "nameTerm": "hero_name",
          "traitUpgrade": { "name": "Hero_Trait_Thorn", "level": 3 }
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        let trait_up = hero.traitUpgrade.as_ref().expect("traitUpgrade should be present");
        assert_eq!(trait_up.name, TRAIT_THORN_CODE);
        assert_eq!(trait_up.level, 3);
    }

    #[test]
    fn hero_definition_round_trips_all_four_new_trait_upgrades() {
        let trait_codes = [
            TRAIT_AXE_THROWER_CODE,
            TRAIT_CHEAPER_CLASS_CODE,
            TRAIT_REGENERATIVE_CODE,
            TRAIT_THORN_CODE,
        ];
        for (idx, &code) in trait_codes.iter().enumerate() {
            let hero = HeroDefinition {
                id: (30 + idx) as i32,
                nameTerm: "hero_name".to_string(),
                traitUpgrade: Some(SerializableHeroUpgrade {
                    name: code.to_string(),
                    level: 1,
                }),
                ..Default::default()
            };
            let json_val = serde_json::to_value(&hero).expect("encode should succeed");
            let decoded: HeroDefinition =
                serde_json::from_value(json_val).expect("decode should succeed");
            let trait_up = decoded.traitUpgrade.as_ref().expect("traitUpgrade should survive round-trip");
            assert_eq!(trait_up.name, code, "trait code mismatch for {}", code);
            assert_eq!(trait_up.level, 1);
        }
    }

    #[test]
    fn hero_definition_trait_upgrade_via_snake_case_alias() {
        // Verify that `trait_upgrade` (snake_case) is accepted as an alias.
        let json = r#"{
          "id": 35,
          "nameTerm": "hero_name",
          "trait_upgrade": { "name": "Hero_Trait_Thorn", "level": 2 }
        }"#;
        let hero: HeroDefinition = serde_json::from_str(json).expect("decode should succeed");
        let trait_up = hero.traitUpgrade.as_ref().expect("traitUpgrade should be present via alias");
        assert_eq!(trait_up.name, TRAIT_THORN_CODE);
        assert_eq!(trait_up.level, 2);
    }
}
