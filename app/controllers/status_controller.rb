class StatusController < ApplicationController

  def status
    render json: {
        up: true
    }
  end

end
