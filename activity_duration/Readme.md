# Activity duration

This crate implements algorithm which counts duration of user activity (e.g. running) by using heart rate.

Currently any heart rate above threshold will count as activity time, but in future this area should be improved,

## How

As input we take heart rate array and user age with resting heart rate.

Algorithm have heart rate ares and qualifies some as resting or exercising.

Current areas:

- **VO2**
- **Anaerobic**
- **Aerobic**
- FatBurn
- WarmUp
- Resting

_Note_: bold items count as exercising.
