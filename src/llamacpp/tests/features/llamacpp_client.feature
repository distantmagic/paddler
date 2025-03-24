Feature: Llama.cpp client behaviour
    Testing the response from the `get_available_slots`
    in different scenarios

  Scenario: Slots endpoint working with authorized slots
    Given llamacpp server is running
    When I request available slots with a authorized response
    Then I should receive a successful response with slots

  Scenario: Slots endpoint working with unathorized slots
    Given llamacpp server is running
    When I request available slots with an unauthorized response
    Then I should receive an unauthorized response

  Scenario: Slots endpoint not working with unimplemented response
    Given llamacpp server is running
    When I request available slots with a not implemented response
    Then I should receive a not implemented response

  Scenario: Slots endpoint not working with error from endpoint
    Given llamacpp server is running
    When I request available slots with an error response
    Then I should receive an error