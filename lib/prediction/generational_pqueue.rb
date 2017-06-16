require_relative './pqueue'

module Prediction

  # manages a series of pqueues, each with a global cost
  # designed to work with the algorithm's generations system
  class GenerationalQueue

    def initialize
      @costs = {}
      @pqueues = {}
    end

    def set_cost(generation, cost)
      @costs[generation] = cost
    end

    def enqueue(generation, element, priority=0, default_cost=0.1)
      @pqueues[generation] = PriorityQueue.new unless @pqueues.has_key? generation
      @costs[generation] = default_cost unless @costs.has_key? generation

      pq = @pqueues[generation]
      pq.push(element, priority)
    end

    # gets the lowest cost item from each pqueue, adds the
    # generation cost to each, then returns the global minimum
    def dequeue
      min = @pqueues.each.select{|tuple| !tuple.last.empty? }.min_by do |tuple|
        generation = tuple.first
        tuple.last.min[1] + @costs[generation]
      end

      return nil if min.blank?

      min.last.delete_min[0]
    end

    def has_elements?
      @pqueues.values.each do |pq|
        return true if !pq.empty?
      end
      false
    end

    def empty?
      !has_elements?
    end

    def size
      @pqueues.sum do |tuple|
        tuple.last.size
      end
    end
  end
end