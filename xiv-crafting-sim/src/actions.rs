use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ActionType {
    Immediate,
    CountUp,
    Countdown {
        active_turns: i32, // number of turns this countdown is active for
    },
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::Immediate
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Combo {
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActionDetails<'a> {
    pub short_name: &'a str,
    pub full_name: &'a str,
    pub durability_cost: i32,
    pub cp_cost: i32,
    pub success_probability: f64,
    pub quality_increase_multiplier: f64,
    pub progress_increase_multiplier: f64,
    pub action_type: ActionType, // action types
    pub class: &'a str,
    pub level: i32,
    pub on_good: bool,
    pub on_excellent: bool,
    pub combo: Option<Combo>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Observe,
    BasicSynth,
    BasicSynth2,
    CarefulSynthesis,
    RapidSynthesis,
    BasicTouch,
    StandardTouch,
    HastyTouch,
    ByregotsBlessing,
    MastersMend,
    TricksOfTheTrade,
    InnerQuiet,
    Manipulation,
    WasteNot,
    WasteNot2,
    Veneration,
    Innovation,
    GreatStrides,
    PreciseTouch,
    MuscleMemory,
    RapidSynthesis2,
    PrudentTouch,
    FocusedSynthesis,
    FocusedTouch,
    Reflect,
    PreparatoryTouch,
    Groundwork,
    DelicateSynthesis,
    IntensiveSynthesis,
    TrainedEye,
    CarefulSynthesis2,
    Groundwork2,
    AdvancedTouch,
    PrudentSynthesis,
    TrainedFinesse,
    Edit,
    FocusedTouchCombo,
    FocusedSynthesisCombo,
}

impl Action {
    pub const fn details(&self) -> ActionDetails {
        match self {
            Action::BasicSynth => {
                ActionDetails {
                    //             shortName	              fullName	        dur	cp	Prob	 QIM	 PIM	 Type	          t	  cls	           lvl
                    //          'basicSynth'	           'Basic Synthesis'	10	0	1	0	1	 'immediate'	1	  'All'	           1)
                    short_name: "basicSynth",
                    full_name: "Basic Synthesis",
                    durability_cost: 10,
                    cp_cost: 0,
                    success_probability: 1.0,
                    quality_increase_multiplier: 0.0,
                    progress_increase_multiplier: 1.0,
                    action_type: ActionType::Immediate,
                    class: "All",
                    level: 1,

                    on_good: false,
                    on_excellent: false,
                    combo: None
                }
            },
            Action::CarefulSynthesis => {
                ActionDetails {
                    short_name: "carefulSynthesis",
                    full_name: "Careful Synthesis",
                    durability_cost: 10,
                    cp_cost: 7,
                    success_probability: 1.0,
                    quality_increase_multiplier: 0.0,
                    progress_increase_multiplier: 1.2,
                    action_type: ActionType::Immediate,
                    class: "All",
                    level: 62,
                    on_good: false,
                    on_excellent: false,
                    combo: None
                }
            },
            Action::BasicTouch => {
                ActionDetails {
                    short_name: "basicTouch",
                    full_name: "Basic Touch",
                    durability_cost: 10,
                    cp_cost: 18,
                    success_probability: 1.0,
                    quality_increase_multiplier: 1.0,
                    progress_increase_multiplier: 0.0,
                    action_type: ActionType::Immediate,
                    class: "All",
                    level: 18,
                    on_good: false,
                    on_excellent: false,
                    combo: None
                }
            }
            _ => ActionDetails {
                short_name: "",
                full_name: "",
                durability_cost: 0,
                cp_cost: 0,
                success_probability: 0.0,
                quality_increase_multiplier: 0.0,
                progress_increase_multiplier: 0.0,
                action_type: ActionType::Immediate,
                class: "",
                level: 0,
                on_good: false,
                on_excellent: false,
                combo: None,
            },
        }
    }
}

/* TABLE COPIED FROM actions.js
var AllActions = {
//                              shortName,              fullName,              dur,     cp, Prob, QIM, PIM, Type,          t,  cls,           lvl,  onGood,     onExcl,      onPoor,    isCombo,    comboName1,     comboName2
observe: new Action(            'observe',              'Observe',               0,      7,  1.0, 0.0, 0.0, 'immediate',   1,  'All',          13),

basicSynth: new Action(         'basicSynth',           'Basic Synthesis',      10,      0,  1.0, 0.0, 1.0, 'immediate',   1,  'All',           1),
basicSynth2: new Action(        'basicSynth2',          'Basic Synthesis',      10,      0,  1.0, 0.0, 1.2, 'immediate',   1,  'All',          31),
carefulSynthesis: new Action(   'carefulSynthesis',     'Careful Synthesis',    10,      7,  1.0, 0.0, 1.5, 'immediate',   1,  'All',          62),
rapidSynthesis: new Action(     'rapidSynthesis',       'Rapid Synthesis',      10,      0,  0.5, 0.0, 2.5, 'immediate',   1,  'All',           9),

basicTouch: new Action(         'basicTouch',           'Basic Touch',          10,     18,  1.0, 1.0, 0.0, 'immediate',   1,  'All',           5),
standardTouch: new Action(      'standardTouch',        'Standard Touch',       10,     32,  1.0, 1.25,0.0, 'immediate',   1,  'All',          18),
hastyTouch: new Action(         'hastyTouch',           'Hasty Touch',          10,      0,  0.6, 1.0, 0.0, 'immediate',   1,  'All',           9),
byregotsBlessing: new Action(   'byregotsBlessing',     'Byregot\'s Blessing',  10,     24,  1.0, 1.0, 0.0, 'immediate',   1,  'All',          50),

mastersMend: new Action(        'mastersMend',          'Master\'s Mend',        0,     88,  1.0, 0.0, 0.0, 'immediate',   1,  'All',           7),
tricksOfTheTrade: new Action(   'tricksOfTheTrade',     'Tricks of the Trade',   0,      0,  1.0, 0.0, 0.0, 'immediate',   1,  'All',          13,  true,       true),

innerQuiet: new Action(         'innerQuiet',           'Inner Quiet',           0,     18,  1.0, 0.0, 0.0, 'countup',     1,  'All',          11),
manipulation: new Action(       'manipulation',         'Manipulation',          0,     96,  1.0, 0.0, 0.0, 'countdown',   8,  'All',          65),
wasteNot: new Action(           'wasteNot',             'Waste Not',             0,     56,  1.0, 0.0, 0.0, 'countdown',   4,  'All',          15),
wasteNot2: new Action(          'wasteNot2',            'Waste Not II',          0,     98,  1.0, 0.0, 0.0, 'countdown',   8,  'All',          47),
veneration: new Action(         'veneration',           'Veneration',            0,     18,  1.0, 0.0, 0.0, 'countdown',   4,  'All',          15),
innovation: new Action(         'innovation',           'Innovation',            0,     18,  1.0, 0.0, 0.0, 'countdown',   4,  'All',          26),
greatStrides: new Action(       'greatStrides',         'Great Strides',         0,     32,  1.0, 0.0, 0.0, 'countdown',   3,  'All',          21),

// Heavensward actions
preciseTouch: new Action(       'preciseTouch',         'Precise Touch',        10,     18,  1.0, 1.5, 0.0, 'immediate',   1,  'All',          53,  true,       true),
muscleMemory: new Action(       'muscleMemory',         'Muscle Memory',        10,      6,  1.0, 0.0, 3.0, 'countdown',   5,  'All',          54),

// Stormblood actions
rapidSynthesis2: new Action(    'rapidSynthesis2',      'Rapid Synthesis',      10,      0,  0.5, 0.0, 5.0, 'immediate',   1,  'All',          63),
prudentTouch: new Action(       'prudentTouch',         'Prudent Touch',         5,     25,  1.0, 1.0, 0.0, 'immediate',   1,  'All',          66),
focusedSynthesis: new Action(   'focusedSynthesis',     'Focused Synthesis',    10,      5,  0.5, 0.0, 2.0, 'immediate',   1,  'All',          67),
focusedTouch: new Action(       'focusedTouch',         'Focused Touch',        10,     18,  0.5, 1.5, 0.0, 'immediate',   1,  'All',          68),
reflect: new Action(            'reflect',              'Reflect',              10,     6,  1.0, 1.0, 0.0, 'immediate',   1,  'All',          69),

// ShadowBringers actions
preparatoryTouch: new Action(   'preparatoryTouch',     'Preparatory Touch',    20,     40,  1.0, 2.0, 0.0, 'immediate',   1,  'All',          71),
groundwork: new Action(         'groundwork',           'Groundwork',           20,     18,  1.0, 0.0, 3.0, 'immediate',   1,  'All',          72),
delicateSynthesis: new Action(  'delicateSynthesis',    'Delicate Synthesis',   10,     32,  1.0, 1.0, 1.0, 'immediate',   1,  'All',          76),
intensiveSynthesis: new Action( 'intensiveSynthesis',   'Intensive Synthesis',  10,      6,  1.0, 0.0, 4.0, 'immediate',   1,  'All',          78,  true,       true),
trainedEye: new Action(         'trainedEye',           'Trained Eye',          10,    250,  1.0, 0.0, 0.0, 'immediate',   1,  'All',          80),

// Endwalker
carefulSynthesis2: new Action(   'carefulSynthesis2',     'Careful Synthesis',  10,      7,  1.0, 0.0, 1.8, 'immediate',   1,  'All',          82),
groundwork2: new Action(         'groundwork2',           'Groundwork',         20,     18,  1.0, 0.0, 3.6, 'immediate',   1,  'All',          86),
advancedTouch: new Action(       'advancedTouch',        'Advanced Touch',      10,     46,  1.0, 1.5, 0.0, 'immediate',   1,  'All',          84),
prudentSynthesis: new Action(    'prudentSynthesis',     'Prudent Synthesis',   5,      18,  1.0, 0.0, 1.8, 'immediate',   1,  'All',          88),
trainedFinesse: new Action(       'trainedFinesse',       'Trained Finesse',    0,      32,  1.0, 1.0, 0.0, 'immediate',   1,  'All',          90),

// Ranged edit: special combo'd actions that are handled differently
// Combo Actions. Making new combo actions need an image, extraActionInfo, and some code in getComboAction() in ffxivcraftmodel.js
// The existence of this breaks the montecarlo simulation but idgaf about that
//                              shortName,              fullName,              dur,     cp, Prob, QIM, PIM, Type,          t,  cls,           lvl,  onGood,     onExcl,      onPoor,    isCombo,    comboName1,     comboName2
focusedTouchCombo: new Action(  'focusedTouchCombo',    'Focused Touch Combo',  10,     25, 1.0,  1.5, 0.0, 'immediate',   1,  'All',         68,   false,      false,       false,     true,       'observe',      'focusedTouch'),
focusedSynthesisCombo: new Action(  'focusedSynthesisCombo',    'Focused Synthesis Combo',  10, 12, 1.0,  0.0, 2.0, 'immediate',   1,  'All',         67,   false,      false,       false,     true,       'observe',      'focusedSynthesis'),


// Special Actions - not selectable
dummyAction: new Action(        'dummyAction',          '______________',        0,      0,  1.0, 0.0, 0.0, 'immediate',   1,  'All',           1)
};
*/
