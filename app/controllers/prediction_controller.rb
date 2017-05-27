class PredictionController < ApplicationController

  def predict
    render json: Predictor.predict(
        lat: 36.8491253,
        lon: -121.4342394,
        altitude: 0,
        profile: 'standard',
        time: 1.hour.from_now,
        burst_altitude: 25000,
        ascent_rate: 5,
        descent_rate: 5
    )
  rescue RuntimeError => e
    NewRelic::Agent.notice_error e
    render json: {
        success: false,
        error: e.to_s
    }, status: 500
  end

end
