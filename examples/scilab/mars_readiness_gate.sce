// Small Scilab-subset example: gate a Mars helicopter launch on battery state.
mars_gravity_m_s2 = 3.71;
target_altitude_m = 3.0;
battery_temperature_c = 12.0;
battery_soc = 0.45;
flight_state = 0;

if battery_temperature_c >= 10.0 then
  if battery_soc > 0.30 then
    flight_state = 1;
  end
end

if flight_state == 1 then
  disp("READY: launch conditions met");
else
  disp("WAIT: battery conditions not met");
end

disp(mars_gravity_m_s2);
disp(target_altitude_m);
