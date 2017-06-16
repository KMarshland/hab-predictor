require_relative './generational_pqueue'

module Prediction

  class << self

    # finds absolute optimal path for num_generations
    # useful only as a testing metric, because it runs in exponential time
    def get_optimal(params)
      get_neighbors = params.with_indifferent_access[:neighbors]
      build_from_finish = params.with_indifferent_access[:build_from_finish]
      start = params.with_indifferent_access[:start]
      num_generations = 5

      current = [start]
      start.previous = nil
      best = start

      num_generations.times do
        previous = Array.new(current)
        previous.each do |prev|
          children = get_neighbors.call(prev)
          children.each do |child|
            unless child.class == NilClass
              child.previous = prev
              if child.lon > best.lon
                best = child
              end
              current << child
            end
          end
        end
      end
      return build_from_finish.call(best) if best.present?
    end

    # heavily modified A* algorithm using a GenerationalPQueue
    # uses a dynamic costing method with generations to prevent the algorithm from getting stuck
    def greedy_search(params)
      #pull stuff out from the params
      get_movement_cost = params.with_indifferent_access[:movement_cost]
      get_heuristic_cost = params.with_indifferent_access[:heuristic]
      get_neighbors = params.with_indifferent_access[:neighbors]

      build_from_finish = params.with_indifferent_access[:build_from_finish]

      start = params.with_indifferent_access[:start]
      finish = params.with_indifferent_access[:finish]

      #in seconds, the maximum time it will check things for
      max_time = params.with_indifferent_access[:timeout]
      start_time = Time.strptime(start.time, '%Y-%m-%dT%H:%M:%SZ')

      num_children = params[:performance] # higher means better performance, lower means faster

      #reset the start of it
      start.previous = nil
      start.movement_cost = 0
      start.cost = 0
      start.visited = true
      start.generation = 0
      start.child_cost = 0

      best_overall = nil
      best_distance = 0

      # a counter that tells you how long it's been since you moved to the next generation
      stagnation = 0

      # tells you the generation number of the next generation
      next_gen = 0

      #keep track of what you're considering, and consider the first one
      considered = GenerationalQueue.new
      considered.enqueue(next_gen, start)

      begin
        #find the lowest cost node to try next until you run out
        while considered.has_elements? do
          current = considered.dequeue

          puts "Visiting #{current.to_string}"

          # increments stagnation if the algorithm hasn't moved on
          if (current.generation + 1) > next_gen
            next_gen = current.generation + 1
            stagnation = 0
          else
            stagnation += 1
          end

          # calculates the cost that will be applied to that generation
          if next_gen == 0 || next_gen == 1
            considered.set_cost next_gen, 0.0
          else
            considered.set_cost next_gen, (-0.01 * stagnation) + 0.1
          end

          #see if you're done
          if current == finish
            puts 'Reached destination'.green
            return build_from_finish.call(current)
          end

          #see if you've reached the timeout
          if max_time.present? && (Time.strptime(current.time, '%Y-%m-%dT%H:%M:%SZ') - start_time).to_f > max_time
            puts "Duration exceeded - guidance finished".yellow
            return build_from_finish.call(best_overall)
          end

          #Look at all the neighbors of that node
          neighbors = get_neighbors.call(current)
          neighbors.each do |next_node|

            next_node.generation = next_node.parent.generation + 1
            movement_cost = get_movement_cost.call(next_node)

            #track the new way to get to it
            next_node.previous = current
            next_node.movement_cost = movement_cost

            heuristic_cost = get_heuristic_cost.call(next_node)
            if best_overall.nil? || next_node.lon > best_distance
              best_overall = next_node
              best_distance = next_node.lon
            end

            youngest = next_node
            child_cost = 0

            if num_children != 0

              num_children.to_i.times do
                children = get_neighbors.call(youngest)
                best_child = nil
                children.each do |child|
                  child.cost = get_heuristic_cost.call(child) + get_movement_cost.call(child)
                  if best_child.nil? || child.cost < best_child.cost
                    best_child = child
                  end
                end

                break if best_child.nil?

                child_cost += best_child.cost
                youngest = best_child
              end

            end

            next_node.child_cost = child_cost

            next_node.cost = heuristic_cost + movement_cost

            #change the priority or add it to the queue
            if next_node.visited
              # considered.change_priority(next_node)
            else
              considered.enqueue(next_node.generation, next_node, (next_node.cost + next_node.child_cost))
            end
          end
        end

        puts 'Ran out of solution space to check'.yellow
      rescue SystemExit, Interrupt
        puts "System interrupt detected (#{(Time.now - start_time).to_f.round(3)} seconds before interrupt)".yellow
      end

      return build_from_finish.call(best_overall) if best_overall.present?
    end

  end

end
