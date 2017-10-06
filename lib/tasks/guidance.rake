
namespace :guidance do
  task :test => [:environment] do
    result = Predictor.guidance(
        lat: 36.8491253,
        lon: -121.4342394,
        altitude: 12000,
        time: 1.hour.from_now,

        duration: 3.days,
        timeout: 30.seconds,
        compare_with_naive: true
    )

    test_output result

    puts
    puts "Got to longitude #{active['longitude']} (only #{naive['longitude']} naively) by #{time}"
  end

  task :test_greenland => [:environment] do
    result = Predictor.guidance(
        lat: 64.1791025,
        lon: -51.7418292,
        altitude: 12000,
        time: 1.hour.from_now,

        duration: 3.days,
        timeout: 10.seconds,
        compare_with_naive: true,

        guidance_type: 'destination',
        destination_lat: 66.364224,
        destination_lon: -38.1470757
    )

    test_output result
  end

  def test_output(result)
    # puts JSON.pretty_generate result
    # puts "\n\n\n"


    naive = (result['naive'].last || {})
    active = (result['positions'].last || {})
    time = active['time']

    puts 'Final position for active guidance'
    puts JSON.pretty_generate active

    puts
    puts 'Final position for naive'
    puts JSON.pretty_generate naive

    puts
    puts 'Metadata'
    puts JSON.pretty_generate result['metadata']
  end

end
