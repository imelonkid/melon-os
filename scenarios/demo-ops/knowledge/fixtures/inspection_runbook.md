# System Inspection Runbook

## Daily Inspection Checklist

1. Check all critical services are running
2. Verify disk usage is below 80%
3. Confirm network connectivity
4. Review error logs from the last 24 hours

## Storage Warning Thresholds

- **Green**: < 70% usage
- **Yellow**: 70-85% usage - monitor closely
- **Red**: > 85% usage - investigate and clean up

## Recommended Actions for Storage Warning

1. Identify largest files and directories
2. Clean up temporary files and old logs
3. Check for orphaned Docker volumes
4. If disk usage remains high, request additional storage

## Network Troubleshooting

1. Check DNS resolution
2. Verify firewall rules
3. Test connectivity to dependent services
4. Review recent network configuration changes
