Feature: Monitoring service behaviour
    Testing if Monitoring service can properly 
    monitor llama.cpp instance in different scenarios

  Scenario: Monitoring service can fetch status
    Given llamacpp 1 server is running
    When monitoring service fetches slots endpoint with an authorized slots response
    Then monitoring service must receive a successful slots response
    When monitoring server reports status
    Then monitoring service must receive a successful report response

  Scenario: Monitoring service can fetch status
    Given llamacpp 1 server is running
    When monitoring service fetches slots endpoint with an unauthorized slots response
    Then monitoring service must receive a successful slots response
    When monitoring server reports status
    Then monitoring service must receive a successful report response

  # Scenario: Monitoring service can fetch status
  #   Given llamacpp 1 server is running
  #   When monitoring service fetches slots endpoint with an unimplemented slots response
  #   Then monitoring service must receive an unimplemented response
  #   When monitoring server reports status
  #   Then monitoring service must receive a successful report response

  # Scenario: Monitoring service can fetch status
  #   Given llamacpp 1 server is running
  #   When monitoring service fetches slots endpoint with an error response
  #   Then monitoring service must receive an error response
  #   When monitoring server reports status
  #   Then monitoring service must receive a successful report response
    