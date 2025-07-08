# Period

**Pre-ovulation**

Temperature remains stable or slightly lower.

**Ovulation**

Identify a 0.3–0.6°C (0.5–1.0°F) rise in Baseline temperature over a few days.
Confirm with Heart Rate Variability (HRV) and sleep changes

**Post-ovulation**

Temperature stays elevated due to progesterone influence

**Period start**

Sudden drop in Baseline temperature signals the onset of menstruation.

## How

As input algorithms accepts HRV, temperature and UNIX timestamp.

1. Iterate over raw data and collect all points for day;
2. Calculate avg of 25% of lowest values for this day;
3. Find absolute difference between base temperature and temperature from step 2;
4. Based on temperature there multiple cases:
4.1 If temperature lower than base and difference bigger than bound consider this end data point;
4.2 If base lower than temperature and difference smaller than `0.7` consider this middle point;
4.3 If temperature is lower than base and difference is smaller than bound and previous point is not middle that was single day before count this as start point. Otherwise this is middle point too;
4.4 If point don't falls in any cases above count this as corrupted point.
5. Collect all points and start iterating over them;
6. Process collected points in the following manner (_note_: this description is simplified due it's complexity):
6.1 If this is start point then count this as `PreOvulation`, but if previous point was `PostOvulation` then this is `PeriodStart`;
6.2 If this is end point and previous point `PostOvulation` then count as `PeriodStart` if no then check X next elements and if they all lower then base count all this X points as `PeriodStart`;
6.3 If this is middle point and last point is `Ovulation` this could be `PostOvulation` or `Ovulation` based on temperature growth. If last `PostOvulation` and there is no rising in temperature then this is `PostOvulation` point. At last check X next points if there temperature growth and select `Ovulation` or `PostOvulation` points;
6.4 If this is corrupted point then ignore it.
7. Return result

## Notes

Currently algorithms accepts HRV, but there is no impact on resulting data. We need to define how exactly HRV should influence and test this on large dataset.

Additionally we need to work more on lifting rules about "corrupted" points. Currently they mostly ignored.
