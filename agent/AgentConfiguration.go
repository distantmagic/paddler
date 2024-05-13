package agent

import "time"

type AgentConfiguration struct {
	ReportingIntervalMiliseconds uint
}

func (self *AgentConfiguration) GetReportingIntervalDuration() time.Duration {
	return time.Duration(self.ReportingIntervalMiliseconds) * time.Millisecond
}
