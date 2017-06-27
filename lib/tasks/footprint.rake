
namespace :footprint do
  task :test => [:environment] do
    puts JSON.pretty_generate(Predictor.footprint(
        lat: 36.8491253,
        lon: -121.4342394,
        altitude: 0,
        time: 1.hour.from_now,

        burst_altitude_mean: 25000,
        burst_altitude_std_dev: 3000,

        ascent_rate_mean: 5,
        ascent_rate_std_dev: 0.5,

        descent_rate_mean: 5,
        descent_rate_std_dev: 0.5,

        trials: 1000
    ))
  end
end
