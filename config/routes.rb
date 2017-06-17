Rails.application.routes.draw do
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  get 'predict' => 'prediction#predict'
  get 'guidance' => 'guidance#guidance'

  root 'status#status'
end
