# Steps by GPS

Algorithm which counts distance traveled by user and counts *potential* steps from traveled distance.

To count distance we use [`Haversine formula`](https://en.wikipedia.org/wiki/Haversine_formula) for 2 points.
In case GPS data contains altitude in both points [`Euclidean distance`](https://en.wikipedia.org/wiki/Euclidean_distance) is used to improve precision.
