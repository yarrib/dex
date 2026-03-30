interface Props {
  choices: { label: string; value: string }[];
  onAction: (action: string, payload?: unknown) => void;
}

export function ChoiceButtons({ choices, onAction }: Props) {
  return (
    <div className="flex flex-wrap gap-2">
      {choices.map((c) => (
        <button
          key={c.value}
          onClick={() => onAction("choice", c.value)}
          className="btn-secondary text-sm"
        >
          {c.label}
        </button>
      ))}
    </div>
  );
}
