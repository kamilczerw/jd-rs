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

type fixture struct {
    LHS  string        `json:"lhs"`
    RHS  string        `json:"rhs"`
    Diff []diffElement `json:"diff"`
}

type scenario struct {
    lhs string
    rhs string
}

var scenarios = map[string]scenario{
    "append": {
        lhs: "[1,2]",
        rhs: "[1,2,3]",
    },
    "removal": {
        lhs: "[1,2,3]",
        rhs: "[1,2]",
    },
    "substitution": {
        lhs: "[1,2,3]",
        rhs: "[1,4,3]",
    },
    "nested_object": {
        lhs: `[{"id":1,"meta":{"name":"jd","version":1}}, {"id":2}]`,
        rhs: `[{"id":1,"meta":{"name":"jd","version":2}}, {"id":2}]`,
    },
    "duplicate_alignment": {
        lhs: "[1,2,1]",
        rhs: "[1,1,2]",
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
    outDir := filepath.Join(root, "crates", "jd-core", "tests", "fixtures", "diff", "list")
    if err := os.MkdirAll(outDir, 0o755); err != nil {
        panic(err)
    }

    names := make([]string, 0, len(scenarios))
    for name := range scenarios {
        names = append(names, name)
    }
    sort.Strings(names)

    for _, name := range names {
        scenario := scenarios[name]
        lhs, err := jd.ReadJsonString(scenario.lhs)
        if err != nil {
            panic(fmt.Errorf("parse lhs for %s: %w", name, err))
        }
        rhs, err := jd.ReadJsonString(scenario.rhs)
        if err != nil {
            panic(fmt.Errorf("parse rhs for %s: %w", name, err))
        }
        diff := lhs.Diff(rhs)
        converted := convertDiff(diff)
        fixture := fixture{
            LHS:  scenario.lhs,
            RHS:  scenario.rhs,
            Diff: converted,
        }
        data, err := json.MarshalIndent(fixture, "", "  ")
        if err != nil {
            panic(err)
        }
        data = append(data, '\n')
        if err := os.WriteFile(filepath.Join(outDir, name+".json"), data, 0o644); err != nil {
            panic(err)
        }
        fmt.Printf("wrote %s\n", filepath.Join(outDir, name+".json"))
    }
}

func findRepoRoot(start string) (string, error) {
    dir := start
    for {
        if _, err := os.Stat(filepath.Join(dir, "crates", "jd-core")); err == nil {
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
        object := make(map[string]nodeRepr, len(v))
        keys := make([]string, 0, len(v))
        for key := range v {
            keys = append(keys, key)
        }
        sort.Strings(keys)
        for _, key := range keys {
            object[key] = convertInterface(v[key])
        }
        return nodeRepr{Type: "Object", Value: object}
    default:
        panic(fmt.Sprintf("unsupported value type %T", v))
    }
}
