class PredictionController < ApplicationController

  require 'prediction/prediction'

  def predict
    if params[:profile] == 'valbal'
      prediction = Predictor.predict(
        lat: params[:lat].to_f,
        lon: params[:lon].to_f,
        altitude: params[:altitude].to_f,
        profile: params[:profile],
        duration: params[:duration],
        time: Time.strptime(params[:time], '%s')
      )
    elsif params[:profile] == 'standard'
      prediction = Predictor.predict(
        lat: params[:lat].to_f,
        lon: params[:lon].to_f,
        altitude: params[:altitude].to_f,
        profile: params[:profile],
        time: Time.strptime(params[:time], '%s'),
        burst_altitude: params[:burst_altitude].to_f,
        ascent_rate: params[:ascent_rate].to_f,
        descent_rate: params[:descent_rate].to_f
      )
    end

    render json: prediction
  rescue RuntimeError => e
    NewRelic::Agent.notice_error e
    render json: {
        success: false,
        error: e.to_s
    }, status: 500
  end

end
