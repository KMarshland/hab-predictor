class PredictionController < ApplicationController

  require 'prediction/prediction'

  def predict
    required_params = [:lat, :lon, :altitude, :time, :profile]

    case params[:profile]
      when 'standard'
        required_params.concat [:ascent_rate, :descent_rate, :burst_altitude]
      when 'valbal'
        required_params.concat [:duration]
      else
        return render json: {
            success: false,
            error: "Invalid profile '#{params[:profile]}'"
        }, status: 400
    end

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
    [:lat, :lon, :altitude, :ascent_rate, :descent_rate, :burst_altitude, :duration].each do |key|
      parameters[key] = parameters[key].to_f if parameters[:key].present?
    end

    render json: Predictor.predict(**parameters)

  rescue RuntimeError => e
    NewRelic::Agent.notice_error e
    render json: {
        success: false,
        error: e.to_s
    }, status: 500
  end

end
