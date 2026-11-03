{{- define "graize.controllerName" -}}
{{- printf "%s-controller" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "graize.uiName" -}}
{{- printf "%s-ui" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "graize.uiDataName" -}}
{{- printf "%s-ui-data" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "graize.selectorLabels" -}}
app.kubernetes.io/name: controller
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "graize.uiSelectorLabels" -}}
app.kubernetes.io/name: ui
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "graize.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "graize.selectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "graize.uiLabels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "graize.uiSelectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}
