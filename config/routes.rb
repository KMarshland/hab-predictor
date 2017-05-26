Rails.application.routes.draw do

  get 'predict' => 'prediction#predict'

  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html
end
