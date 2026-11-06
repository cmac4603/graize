{{- define "graize.controllerName" -}}
{{- printf "%s-controller" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "graize.frontendName" -}}
{{- printf "%s-frontend" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "graize.frontendDataName" -}}
{{- printf "%s-frontend-data" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "graize.selectorLabels" -}}
app.kubernetes.io/name: controller
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "graize.frontendSelectorLabels" -}}
app.kubernetes.io/name: frontend
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "graize.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "graize.selectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "graize.frontendLabels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "graize.frontendSelectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}
