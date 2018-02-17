
module Downloader

  class Standalone

    OUTPUT_FILE = Rails.root.join('lib', 'downloader', 'standalone.rb')

    class << self

      def build

        worker = File.read(Rails.root.join('app', 'workers', 'import_worker.rb'))
        datasets_lib = File.read(Rails.root.join('lib', 'storage', 'processed_datasets.rb'))
        script = File.read(Rails.root.join('lib', 'storage', 'importer_script.rb')).gsub("require_relative 'processed_datasets'", '')


        output = [
            "require 'date'",
            "require 'active_support/all'",
            "require 'azure/storage'",

            [
                "require 'pathname'",
                "class Rails",
                "def self.root; Pathname.new('.'); end",
                "end"
            ].join("\n"),

            worker,
            datasets_lib,
            script
        ]

        File.write(OUTPUT_FILE, output.join("\n\n"))

        puts "Built to #{OUTPUT_FILE}"

      end

    end

  end

end
