# Owner decisions needed

- The plan fixes alert thresholds and gives exact message copy for the 20% left, fast drain and org hard stop examples. It does not give approved text for 50% left, 10% left, or a reached limit without org overage disabled. Task 5 currently uses the same template as the approved 20% example; confirm or replace those strings before a release.
- The fast drain message example says “12% used in 7 min”, but `WindowView` exposes only a rate, not the measured span. Task 5 currently reports the equivalent five-minute rate. Confirm whether the message needs the actual observation interval and, if so, add that field to the approved interface.
