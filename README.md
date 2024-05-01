# dnsmonitor
Tool written in Rust

## Getting started
dnsmonitor -s 999 www.heise.de ldap-db-tkom.tv SOABP-PRD.de.ai
- -d for debug output

### Metrics
On port 8080 /metrics

Example:
```
# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter Custom metric returning the failures per dns name.
# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter gauge
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter{dns="ldap-db-tkom.tv"} 0
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter{dns="www.heise.de"} 4
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_failurecounter{dns="SOABP-PRD.de.ai"} 0
# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min Custom metric returning the min time in microseconds per dns name.
# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min gauge
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min{dns="ldap-db-tkom.tv"} 637
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min{dns="www.heise.de"} 586
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_min{dns="SOABP-PRD.de.ai"} 484
# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max Custom metric returning the max time in microseconds per dns name.
# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max gauge
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max{dns="ldap-db-tkom.tv"} 1158
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max{dns="www.heise.de"} 82415
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_max{dns="SOABP-PRD.de.ai"} 1121
# HELP yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average Custom metric returning the average time in microseconds per dns name.
# TYPE yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average gauge
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average{dns="ldap-db-tkom.tv"} 922
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average{dns="www.heise.de"} 30243
yodaforce_application_de_codecoverage_crm_dns_monitor_dnsMonitor_average{dns="SOABP-PRD.de.ai"} 932
```

#### SANITIZER:
rustup target list
RUSTFLAGS="-Z sanitizer=thread" cargo run +nightly --target x86_64-pc-windows-msvc
