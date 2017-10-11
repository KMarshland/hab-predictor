
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

    destination = {
        latitude: 66.364224,
        longitude: -38.1470757
    }

    result = Predictor.guidance(
        lat: 64.1791025,
        lon: -51.7418292,
        altitude: 12000,
        time: 1.hour.from_now,
        time_increment: 30,

        duration: 3.days,
        timeout: 10.seconds,
        compare_with_naive: true,

        guidance_type: 'destination',
        destination_lat: destination[:latitude],
        destination_lon: destination[:longitude]
    )

    test_output result

    naive = (result['naive'].last || {}).symbolize_keys
    active = (result['positions'].last || {}).symbolize_keys

    naive_distance = distance(destination, naive)
    guided_distance = distance(destination, active)

    puts
    puts "Got within #{guided_distance.round}km (#{naive_distance.round}km naively)"

  end

  task :test_distance do
    output = distance(
        {
            latitude: 64.1791025,
            longitude: -51.741829
        },
        {
            latitude: 66.364224,
            longitude: -38.1470757
        }
    )

    puts "Calculated #{output.round}km, expected #{676}km"
  end

  # distance in kilometers between coordinates
  def distance(from, to, earth_radius: 6_371)
    rad_per_deg = Math::PI / 180

    lat1 = from[:latitude].to_f * rad_per_deg
    lon1 = from[:longitude].to_f * rad_per_deg
    lat2 = to[:latitude].to_f * rad_per_deg
    lon2 = to[:longitude].to_f * rad_per_deg

    delta_lat = lat2 - lat1
    delta_lon = lon2 - lon1

    a = Math.sin(delta_lat/2) * Math.sin(delta_lat/2) +
        Math.cos(lat1) * Math.cos(lat2) *
            Math.sin(delta_lon/2) * Math.sin(delta_lon/2)
    c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a))

    earth_radius*c
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
