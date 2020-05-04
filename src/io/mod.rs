
enum InputState
{
    Unknown,
    Low,
    High,
    Short,
    Cutout
}

// The interface of Input providing modules towards
// the IO module. All changes are propagated this way
struct RawInputEvent
{
    input_id: u32,      // SUD!
    state: InputState
}


// Interface of the IO Module to the rest of the
// system. Logical Input states, which have been
// debounce appropiately are propagated this way
struct InputEvent
{
    input_id: u32,      // Logical!
    state: InputState
}

/// # InputSetting
/// This struct describes a runtimesetting for
/// a given digital input. The fiels have the
/// following semantics:
/// * input_id: Contains the logical id of the 
///             input, which was derived from the
///             SUD.
/// * inverted_polarity: Controls if the input is
///             considered to be active low. If
///             set to true, a physical state of 
///             "Low" will be inverted to "High"
///             and vice versa
/// * The debouncetimes (in ms) control how long a given signal must not change, before an InputEvent is triggered.
struct InputSetting
{
    input_id: u32,              //Logical?
    inverted_polarity: bool,
    debounce_on: u64,
    debounce_off: u64
}

enum OutputState
{
    Low,
    High
}

struct RawOutputSwitch
{
    output_id: u32,     // SUD!
    target_state: OutputState   // physical!
}

struct OutputSwitch
{
    output_id: u32,
    target_state: OutputState,   //logical!
    switch_time: u64            // in ms!
}