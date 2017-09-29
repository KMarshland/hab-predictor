
is_rake = !ARGV.any?{ |arg|
  arg =~ /puma/
}

unless is_rake

  Process.fork do
    require Rails.root.join('lib', 'storage', 'importer_script.rb')
  end

end
