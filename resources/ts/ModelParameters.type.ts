export type ModelParameters = {
  batch_n_tokens: number;
  context_size: number;
  min_p: number;
  penalty_frequency: number;
  penalty_last_n: number;
  penalty_presence: number;
  penalty_repeat: number;
  temperature: number;
  top_k: number;
  top_p: number;
};
