class FootprintController < ApplicationController

  def footprint
    required_params = [
        :lat, :lon, :altitude, :time,
        :burst_altitude_mean, :burst_altitude_std_dev,
        :ascent_rate_mean, :ascent_rate_std_dev,
        :descent_rate_mean, :descent_rate_std_dev,
        :trials
    ]

    parameters = {}
    missing = []
    required_params.each do |key|
      parameters[key] = params[key]
      missing << key if params[key].blank?
    end

    if missing.any?
      return render json: {
          success: false,
          error: "Missing required parameters: #{missing.join(', ')}"
      }, status: 400
    end

    parameters[:time] = DateTime.strptime(parameters[:time], '%s')
    [
        :lat, :lon, :altitude,
        :burst_altitude_mean, :burst_altitude_std_dev,
        :ascent_rate_mean, :ascent_rate_std_dev,
        :descent_rate_mean, :descent_rate_std_dev,
        :trials
    ].each do |key|
      parameters[key] = parameters[key].to_f if parameters[:key].present?
    end

    render json: Predictor.footprint(**parameters)

  rescue RuntimeError => e
    NewRelic::Agent.notice_error e
    render json: {
        success: false,
        error: e.to_s
    }, status: 500
  end

end
