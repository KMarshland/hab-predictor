class GuidanceController < ApplicationController

  def guidance

    errors = []

    valid_and_present = lambda {|key|
      params[key].present? && params[key] != 'undefined'
    }

    if valid_and_present.call :latitude
      latitude = params[:latitude].to_f
    else
      errors << 'Missing latitude field.'
    end

    if valid_and_present.call :longitude
      longitude = params[:longitude].to_f
    else
      errors << 'Missing longitude field.'
    end

    if valid_and_present.call :altitude
      altitude = params[:altitude].to_f
    else
      errors << 'Missing altitude field.'
    end

    if valid_and_present.call :performance
      performance = params[:performance].to_f
    else
      errors << 'Missing performance field.'
    end

    if valid_and_present.call :start_time
      begin
        start_time = Time.strptime(params[:start_time], '%s')
      rescue ArgumentError => e
        errors << 'Invalid start time date - Input should be seconds since the UNIX epoch.'
      end
    else
      errors << 'Missing start time field.'
    end

    if valid_and_present.call :end_time
      begin
        end_time = Time.strptime(params[:end_time], '%s')
      rescue ArgumentError => e
        errors << 'Invalid end time date - Input should be seconds since the UNIX epoch.'
      end
    else
      errors << 'Missing end time field.'
    end

    latest = Prediction::latest_dataset

    if errors.blank?
      if latest.nil?
        errors << 'No datasets found - they are likely still being downloaded.'
      elsif end_time > (latest + (180 * 60 * 60).seconds - (performance * 60 * Prediction::time_variance).seconds)
        errors << 'End time outside prediction range.'
      elsif start_time > end_time
        errors << 'Start time must be earlier than end time.'
      end
    end

    if errors.blank?

      my_params = {
          start: {
              lat: latitude,
              lon: longitude,
              altitude: altitude,
              time: start_time.to_i
          },
          timeout: end_time - start_time,
          performance: performance,
          use_faa: false,
          countries: false
      }

      optimized = Prediction::maximize_distance my_params
      puts "Generated optimized (#{optimized.length} segments)".green

      render json: {
          adjustments: optimized
      }
    else
      render json: {
          success: false,
          error: errors
      }, status: 500
    end

  # Last resort, fallback error handling
  rescue RuntimeError => e
    NewRelic::Agent.notice_error e
    render json: {
        success: false,
        error: e.to_s
    }, status: 500
  end

end
