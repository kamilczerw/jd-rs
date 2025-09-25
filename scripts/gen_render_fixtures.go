package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"

	jd "github.com/josephburnett/jd/v2"
)

type nodeRepr struct {
	Type  string      `json:"type"`
	Value interface{} `json:"value,omitempty"`
}

type diffMetadata struct {
	Merge bool `json:"merge"`
}

type diffElement struct {
	Metadata *diffMetadata `json:"metadata,omitempty"`
	Path     []interface{} `json:"path"`
	Before   []nodeRepr    `json:"before,omitempty"`
	Remove   []nodeRepr    `json:"remove,omitempty"`
	Add      []nodeRepr    `json:"add,omitempty"`
	After    []nodeRepr    `json:"after,omitempty"`
}

type renderOutputs struct {
	Native      string `json:"native,omitempty"`
	NativeColor string `json:"native_color,omitempty"`
	Patch       string `json:"patch,omitempty"`
	Merge       string `json:"merge,omitempty"`
}

type fixture struct {
	Name    string        `json:"name"`
	LHS     string        `json:"lhs"`
	RHS     string        `json:"rhs"`
	Options []string      `json:"options,omitempty"`
	Diff    []diffElement `json:"diff"`
	Render  renderOutputs `json:"render"`
}

type scenario struct {
	name       string
	lhs        string
	rhs        string
	options    []string
	wantNative bool
	wantColor  bool
	wantPatch  bool
	wantMerge  bool
}

var scenarios = []scenario{
	{
		name:       "object_update",
		lhs:        `{"a":1,"b":2}`,
		rhs:        `{"a":2,"b":3}`,
		wantNative: true,
		wantPatch:  true,
	},
	{
		name:       "string_diff_color",
		lhs:        `"kitten"`,
		rhs:        `"sitting"`,
		wantNative: true,
		wantColor:  true,
		wantPatch:  true,
	},
	{
		name:       "list_append",
		lhs:        `[1,2]`,
		rhs:        `[1,2,3,4]`,
		wantNative: true,
		wantPatch:  true,
	},
	{
		name:       "merge_object",
		lhs:        `{"config":{"enabled":false}}`,
		rhs:        `{"config":{"enabled":true,"threshold":5}}`,
		options:    []string{"merge"},
		wantNative: true,
		wantMerge:  true,
	},
}

func main() {
	cwd, err := os.Getwd()
	if err != nil {
		panic(err)
	}
	root, err := findRepoRoot(cwd)
	if err != nil {
		panic(err)
	}
	outDir := filepath.Join(root, "crates", "jd-core", "tests", "fixtures", "render")
	if err := os.MkdirAll(outDir, 0o755); err != nil {
		panic(err)
	}

	names := make([]string, len(scenarios))
	for i, scenario := range scenarios {
		names[i] = scenario.name
	}
	sort.Strings(names)

	byName := make(map[string]scenario)
	for _, scenario := range scenarios {
		byName[scenario.name] = scenario
	}

	for _, name := range names {
		scenario := byName[name]
		lhs, err := readNode(scenario.lhs)
		if err != nil {
			panic(fmt.Errorf("parse lhs for %s: %w", name, err))
		}
		rhs, err := readNode(scenario.rhs)
		if err != nil {
			panic(fmt.Errorf("parse rhs for %s: %w", name, err))
		}
		options := convertOptions(scenario.options)
		diff := lhs.Diff(rhs, options...)
		convertedDiff := convertDiff(diff)

		outputs := renderOutputs{}
		if scenario.wantNative {
			outputs.Native = diff.Render()
		}
		if scenario.wantColor {
			outputs.NativeColor = diff.Render(jd.COLOR)
		}
		if scenario.wantPatch {
			str, err := diff.RenderPatch()
			if err != nil {
				panic(fmt.Errorf("render patch for %s: %w", name, err))
			}
			outputs.Patch = str
		}
		if scenario.wantMerge {
			str, err := diff.RenderMerge()
			if err != nil {
				panic(fmt.Errorf("render merge for %s: %w", name, err))
			}
			outputs.Merge = str
		}

		data := fixture{
			Name:    scenario.name,
			LHS:     scenario.lhs,
			RHS:     scenario.rhs,
			Options: scenario.options,
			Diff:    convertedDiff,
			Render:  outputs,
		}

		encoded, err := json.MarshalIndent(data, "", "  ")
		if err != nil {
			panic(err)
		}
		encoded = append(encoded, '\n')
		outPath := filepath.Join(outDir, scenario.name+".json")
		if err := os.WriteFile(outPath, encoded, 0o644); err != nil {
			panic(err)
		}
		fmt.Printf("wrote %s\n", outPath)
	}
}

func readNode(input string) (jd.JsonNode, error) {
	node, err := jd.ReadJsonString(input)
	if err != nil {
		return nil, err
	}
	return node, nil
}

func convertOptions(opts []string) []jd.Option {
	converted := make([]jd.Option, 0, len(opts))
	for _, opt := range opts {
		switch opt {
		case "merge":
			converted = append(converted, jd.MERGE)
		case "set":
			converted = append(converted, jd.SET)
		case "mset":
			converted = append(converted, jd.MULTISET)
		default:
			panic(fmt.Sprintf("unsupported option %q", opt))
		}
	}
	return converted
}

func findRepoRoot(start string) (string, error) {
	dir := start
	for {
		marker := filepath.Join(dir, "crates", "jd-core")
		if _, err := os.Stat(marker); err == nil {
			return dir, nil
		}
		next := filepath.Dir(dir)
		if next == dir {
			return "", fmt.Errorf("could not locate repo root from %s", start)
		}
		dir = next
	}
}

func convertDiff(diff jd.Diff) []diffElement {
	elements := make([]diffElement, len(diff))
	for i, element := range diff {
		var metadata *diffMetadata
		if element.Metadata.Merge {
			metadata = &diffMetadata{Merge: true}
		}
		elements[i] = diffElement{
			Metadata: metadata,
			Path:     convertPath(element.Path),
			Before:   convertNodes(element.Before),
			Remove:   convertNodes(element.Remove),
			Add:      convertNodes(element.Add),
			After:    convertNodes(element.After),
		}
	}
	return elements
}

func convertPath(path jd.Path) []interface{} {
	segments := make([]interface{}, len(path))
	for i, segment := range path {
		switch v := segment.(type) {
		case jd.PathKey:
			segments[i] = string(v)
		case jd.PathIndex:
			segments[i] = int(v)
		default:
			panic(fmt.Sprintf("unsupported path element %T", v))
		}
	}
	return segments
}

func convertNodes(nodes []jd.JsonNode) []nodeRepr {
	if len(nodes) == 0 {
		return []nodeRepr{}
	}
	converted := make([]nodeRepr, len(nodes))
	for i, node := range nodes {
		converted[i] = convertNode(node)
	}
	return converted
}

func convertNode(node jd.JsonNode) nodeRepr {
	rendered := node.Json()
	if rendered == "" {
		return nodeRepr{Type: "Void"}
	}
	var raw interface{}
	if err := json.Unmarshal([]byte(rendered), &raw); err != nil {
		panic(err)
	}
	return convertInterface(raw)
}

func convertInterface(value interface{}) nodeRepr {
	switch v := value.(type) {
	case nil:
		return nodeRepr{Type: "Null"}
	case bool:
		return nodeRepr{Type: "Bool", Value: v}
	case float64:
		return nodeRepr{Type: "Number", Value: v}
	case string:
		return nodeRepr{Type: "String", Value: v}
	case []interface{}:
		children := make([]nodeRepr, len(v))
		for i, child := range v {
			children[i] = convertInterface(child)
		}
		return nodeRepr{Type: "Array", Value: children}
	case map[string]interface{}:
		keys := make([]string, 0, len(v))
		for key := range v {
			keys = append(keys, key)
		}
		sort.Strings(keys)
		children := make(map[string]nodeRepr, len(v))
		for _, key := range keys {
			children[key] = convertInterface(v[key])
		}
		return nodeRepr{Type: "Object", Value: children}
	default:
		panic(fmt.Sprintf("unsupported value type %T", v))
	}
}
