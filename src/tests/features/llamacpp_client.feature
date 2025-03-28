Feature: Llama.cpp client behaviour
    Testing the response from the `get_available_slots`
    in different scenarios

  Scenario: Slots endpoint working with authorized slots
    Given llamacpp server is running
    When llamacpp client requests available slots endpoint with an authorized slots response
    Then llamacpp client must receive a successful response with slots

  Scenario: Slots endpoint working with unathorized slots
    Given llamacpp server is running
    When llamacpp client requests available slots endpoint with an unauthorized slots response
    Then llamacpp client must receive an unauthorized response

  Scenario: Slots endpoint not working with unimplemented response
    Given llamacpp server is running
    When llamacpp client requests available slots endpoint with an not implemented response
    Then llamacpp client must receive an unimplemented response

  Scenario: Slots endpoint not working with error from endpoint
    Given llamacpp server is running
    When llamacpp client requests available slots endpoint with an error response
    Then llamacpp client must receive an error reponse