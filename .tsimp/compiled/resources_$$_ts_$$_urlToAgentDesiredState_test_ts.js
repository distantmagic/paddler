import test from 'ava';
import { urlToAgentDesiredState } from './urlToAgentDesiredState';
test('recognizes Hugging Face urls', function (test) {
    const url = new URL("https://huggingface.co/Qwen/Qwen3-0.6B-GGUF/blob/main/Qwen3-0.6B-Q8_0.gguf");
    test.deepEqual(urlToAgentDesiredState(url), {
        model: {
            HuggingFace: {
                filename: "Qwen3-0.6B-Q8_0.gguf",
                repo: "Qwen/Qwen3-0.6B-GGUF",
            }
        }
    });
});
test('uses local urls', function (test) {
    const url = new URL("file:///home/user/models/Qwen3-0.6B-Q8_0.gguf");
    test.deepEqual(urlToAgentDesiredState(url), {
        model: {
            Local: "/home/user/models/Qwen3-0.6B-Q8_0.gguf"
        }
    });
});
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidXJsVG9BZ2VudERlc2lyZWRTdGF0ZV90ZXN0LmpzIiwic291cmNlUm9vdCI6Ii9ob21lL21jaGFyeXRvbml1ay93b3Jrc3BhY2UvaW50ZW50ZWUvcGFkZGxlci8iLCJzb3VyY2VzIjpbInJlc291cmNlcy90cy91cmxUb0FnZW50RGVzaXJlZFN0YXRlX3Rlc3QudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxJQUFJLE1BQU0sS0FBSyxDQUFDO0FBQ3ZCLE9BQU8sRUFBRSxzQkFBc0IsRUFBRSxNQUFNLDBCQUEwQixDQUFDO0FBRWxFLElBQUksQ0FBQyw4QkFBOEIsRUFBRSxVQUFVLElBQUk7SUFDakQsTUFBTSxHQUFHLEdBQUcsSUFBSSxHQUFHLENBQUMsNEVBQTRFLENBQUMsQ0FBQztJQUVuRyxJQUFJLENBQUMsU0FBUyxDQUNYLHNCQUFzQixDQUFDLEdBQUcsQ0FBQyxFQUMzQjtRQUNFLEtBQUssRUFBRTtZQUNMLFdBQVcsRUFBRTtnQkFDWCxRQUFRLEVBQUUsc0JBQXNCO2dCQUNoQyxJQUFJLEVBQUUsc0JBQXNCO2FBQzdCO1NBQ0Y7S0FDRixDQUNGLENBQUM7QUFDSixDQUFDLENBQUMsQ0FBQztBQUVILElBQUksQ0FBQyxpQkFBaUIsRUFBRSxVQUFVLElBQUk7SUFDcEMsTUFBTSxHQUFHLEdBQUcsSUFBSSxHQUFHLENBQUMsK0NBQStDLENBQUMsQ0FBQztJQUVyRSxJQUFJLENBQUMsU0FBUyxDQUNaLHNCQUFzQixDQUFDLEdBQUcsQ0FBQyxFQUMzQjtRQUNFLEtBQUssRUFBRTtZQUNMLEtBQUssRUFBRSx3Q0FBd0M7U0FDaEQ7S0FDRixDQUNGLENBQUM7QUFDSixDQUFDLENBQUMsQ0FBQyJ9