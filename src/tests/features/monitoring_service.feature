Feature: Monitoring service behaviour
    Testing if Monitoring service can properly 
    monitor llama.cpp instance in different scenarios

  Scenario: Monitoring service can fetch status
    Given llamacpp 1 server is running
    When monitoring service fetches slots endpoint
    Then monitoring service must receive a successful response
    When monitoring server reports status
    Then monitoring service must receive a successful report response
    