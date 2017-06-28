
namespace :guidance do
  task :test => [:environment] do
    result = Predictor.guidance(
        lat: 36.8491253,
        lon: -121.4342394,
        altitude: 0,
        time: 1.hour.from_now,

        timeout: 10,
        compare_with_naive: true
    )

    puts JSON.pretty_generate result
    puts "\n\n\n"

    naive = (result['naive'].last || {})['longitude']
    active = (result['positions'].last || {})['longitude']

    puts "Got to longitude #{active} (only #{naive} naively)"
  end
end
