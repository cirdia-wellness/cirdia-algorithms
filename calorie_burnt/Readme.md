# Calorie burnt

This crate provides formula to calculate calories burn by activity MET.
This algorithm highly depends on activity types detection and it's precision will drop.

Additionally it gives option to provide custom MET or calculate MET of activity based on heart rate.

Alternatively this crate gives option to use machine learning to predict calories burnt.

## How

Used formula to calculate MET:

```plain
(DURATION * MET * WEIGHT) / 200
```

- *DURATION* - duration of activity in minutes
- *WEIGHT* - weight of person in kilograms
