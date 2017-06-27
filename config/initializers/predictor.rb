# Re-namespace rust predictor
require 'predictor'

Object.const_set('RustPredictor', Predictor)
Object.send(:remove_const, :Predictor)
ENV["RAILS_ROOT"] = Rails.root.to_s

# Define our own version with nice wrapper functions
class Predictor

  class << self

    def test(arg='Hi')
      RustPredictor.test arg
    end

    def predict(lat:, lon:, altitude:, time:, profile:, burst_altitude: nil, ascent_rate: nil, descent_rate: nil, duration: nil)

      unless %w(standard valbal).include? profile
        raise ArgumentError, "Invalid profile '#{profile}'"
      end

      case profile
        when 'standard'
          raise ArgumentError, 'Missing required parameter burst_altitude' if burst_altitude.blank?
          raise ArgumentError, 'Missing required parameter ascent_rate' if ascent_rate.blank?
          raise ArgumentError, 'Missing required parameter descent_rate' if descent_rate.blank?
        when 'valbal'
          raise ArgumentError, 'Missing required parameter duration' if duration.blank?
      end

      JSON(RustPredictor.predict(
                       lat.to_f,
                       lon.to_f,
                       altitude.to_f,
                       time.to_i.to_s,
                       profile.to_s,
                       burst_altitude.to_f,
                       ascent_rate.to_f,
                       descent_rate.to_f,
                       duration.to_f
      ))
    end

    def calculate_footprint(lat:, lon:, altitude:, time:, burst_altitude_mean:, burst_altitude_variance:, ascent_rate_mean:, ascent_rate_variance:, descent_rate_mean:, descent_rate_variance:, trials:)
      JSON(RustPredictor.calculate_footprint(
          lat.to_f,
          lon.to_f,
          altitude.to_f,
          time.to_i.to_s,
          burst_altitude_mean.to_f,
          burst_altitude_variance.to_f,
          ascent_rate_mean.to_f,
          ascent_rate_variance.to_f,
          descent_rate_mean.to_f,
          descent_rate_variance.to_f,
          trials.to_i
      ))
    end

  end

end