Rails.application.routes.draw do
  # For details on the DSL available within this file, see http://guides.rubyonrails.org/routing.html

  get 'predict' => 'prediction#predict'
  get 'footprint' => 'footprint#footprint'
  get 'guidance' => 'guidance#guidance'

  get 'status/datasets'

  root 'status#status'
end
