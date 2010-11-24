//
//  FPLevelView.m
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPLevelView.h"

@implementation FPLevelView

- (id)initWithCoder:(NSCoder *)coder
{
	self = [super initWithCoder:coder];
    if (self) 
	{
		[self setupSharedContext];
	    glClearColor(55.0f / 255.0f, 60.0f / 255.0f, 89.0f / 255.0f, 1.0f);
		glEnable(GL_TEXTURE_2D);
		glEnable(GL_BLEND);
		glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
		beginMovePoint = endMovePoint = CGPointZero;
		currentHandle = FPDragHandleNone;
    }
    return self;
}

- (void)dealloc
{
	[super dealloc];
}

- (BOOL)acceptsFirstResponder
{
	return YES;
}

- (void)awakeFromNib
{
	NSWindow *window = [self window];
	[window setAcceptsMouseMovedEvents:YES];
}

- (BOOL)isFlipped
{
	return YES;
}

- (void)reshape
{
	[self setNeedsDisplay:YES];
}

#pragma mark Mouse Events

- (void)mouseDown:(NSEvent *)theEvent
{
	CGPoint location = [self locationFromNSEvent:theEvent];
	int x = location.x;
	int y = location.y;
	x /= 32;
	x *= 32;
	y /= 32;
	y *= 32;
	beginMovePoint.x = x;
	beginMovePoint.y = y;
	endMovePoint = beginMovePoint;
	
	if (currentHandle != FPDragHandleNone && currentHandle != FPDragHandleCenter)
	{
		[dataSource beginResizeWithHandle:currentHandle];
		return;
	}
	
	id factory = [dataSource activeFactory];
	NSMutableArray *gameObjects = [dataSource gameObjects];
	NSMutableIndexSet *selectedIndices = [dataSource selectedIndices];
	
	if (factory)
	{
		id<FPGameObject> draggedObject = (id<FPGameObject>)[[factory alloc] init];		
		[draggedObject moveWithX:x y:y];
		[dataSource addNewGameObject:draggedObject];
		[draggedObject release];		
	}
	else
	{
		NSUInteger flags = [theEvent modifierFlags];
		BOOL startSelection = YES;
		
		if ((flags & NSCommandKeyMask) == NSCommandKeyMask)
		{
			for (NSUInteger i = 0; i < [gameObjects count]; i++)
			{
				id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
				if (CGRectContainsPoint([gameObject rect], location))
				{
					if ([selectedIndices containsIndex:i])
						[selectedIndices removeIndex:i];
					else
						[selectedIndices addIndex:i];
					
					startSelection = NO;
					break;
				}
			}
		}
		else if ((flags & NSShiftKeyMask) == NSShiftKeyMask)
		{
			for (NSUInteger i = 0; i < [gameObjects count]; i++)
			{
				id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
				if (CGRectContainsPoint([gameObject rect], location))
				{
					[selectedIndices addIndex:i];
					startSelection = NO;
					break;
				}
			}
		}
		else if (currentHandle == FPDragHandleNone)
		{
			for (NSUInteger i = 0; i < [gameObjects count]; i++)
			{
				id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
				if (CGRectContainsPoint([gameObject rect], location))
				{
					[selectedIndices removeAllIndexes];
					[selectedIndices addIndex:i];
					startSelection = NO;
					break;
				}
			}
		}
		else
		{
			startSelection = NO;
			[dataSource beginMove];
		}

		if (startSelection)
		{
			drawingSelection = YES;
			endSelection = beginSelection = location;
		}
	}
	
	[self setNeedsDisplay:YES];
}

- (void)mouseMoved:(NSEvent *)theEvent
{
	CGPoint location = [self locationFromNSEvent:theEvent];
	currentHandle = FPDragHandleNone;
				   
	const float handleSize = 14.0f;
	CGRect handleRect = CGRectMake(0.0f, 0.0f, handleSize, handleSize);
	
	NSMutableArray *gameObjects = [dataSource gameObjects];
	NSMutableIndexSet *selectedIndices = [dataSource selectedIndices];
	
	if ([selectedIndices count] == 1)
	{
		id<FPGameObject> draggedObject = [self draggedObject];
		for (FPDragHandle handle = FPDragHandleTopLeft; handle < FPDragHandleCenter; handle++)
		{
			if (![self respondsGameObject:draggedObject toDragHandle:handle])
				continue;
			
			CGPoint handlePoint = [self pointFromHandle:handle aroundRect:draggedObject.rect];
			
			handleRect.origin.x = handlePoint.x - handleRect.size.width / 2.0f;
			handleRect.origin.y = handlePoint.y - handleRect.size.height / 2.0f;
			
			if (CGRectContainsPoint(handleRect, location))
			{
				currentHandle = handle;
				beginMovePoint = endMovePoint = location;
				break;			
			}
		}
		
		if (currentHandle == FPDragHandleNone)
		{
			if (CGRectContainsPoint(draggedObject.rect, location))
			{
				currentHandle = FPDragHandleCenter;
				beginMovePoint = endMovePoint = location;
			}
		}
	}
	else
	{
		[selectedIndices enumerateIndexesUsingBlock:^(NSUInteger idx, BOOL *stop) 
		{ 
			id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:idx];
			if (CGRectContainsPoint(gameObject.rect, location))
			{
				currentHandle = FPDragHandleCenter;
				beginMovePoint = endMovePoint = location;
				*stop = YES;
			}
		}];
	}
				   
	[self setNeedsDisplay:YES];
}

- (void)mouseDragged:(NSEvent *)theEvent
{
	CGPoint location = [self locationFromNSEvent:theEvent];
	
	if (drawingSelection)
	{
		endSelection = location;
	}
	else if (currentHandle != FPDragHandleNone)
	{
		switch (currentHandle)
		{
			case FPDragHandleTopLeft:
				[self resizeDraggedObjectTop:location.y];
				[self resizeDraggedObjectLeft:location.x];
				break;
			case FPDragHandleTopRight:
				[self resizeDraggedObjectTop:location.y];
				[self resizeDraggedObjectRight:location.x];
				break;
			case FPDragHandleBottomLeft:
				[self resizeDraggedObjectBottom:location.y];
				[self resizeDraggedObjectLeft:location.x];
				break;
			case FPDragHandleBottomRight:
				[self resizeDraggedObjectBottom:location.y];
				[self resizeDraggedObjectRight:location.x];
				break;
			case FPDragHandleMiddleTop:
				[self resizeDraggedObjectTop:location.y];
				break;
			case FPDragHandleMiddleBottom:
				[self resizeDraggedObjectBottom:location.y];
				break;
			case FPDragHandleMiddleLeft:
				[self resizeDraggedObjectLeft:location.x];
				break;
			case FPDragHandleMiddleRight:
				[self resizeDraggedObjectRight:location.x];
				break;
			case FPDragHandleCenter:
				if ([[dataSource selectedIndices] count] == 1)
					[self moveDraggedObjectWithX:location.x y:location.y];
				else
					[self moveSelectedObjectsWithX:location.x y:location.y];
				break;
			default:
				break;
		}
	}
	else if ([dataSource activeFactory])
	{
		int widthSegments = (location.x - endMovePoint.x + 16.0f) / 32.0f;
		int heightSegments = (location.y - endMovePoint.y + 16.0f) / 32.0f;
		
		CGPoint draggedObjectLocation = endMovePoint;
		if (widthSegments < 0)
			draggedObjectLocation.x += widthSegments * 32.0f;
		if (heightSegments < 0)
			draggedObjectLocation.y += heightSegments * 32.0f;

		widthSegments = MAX(ABS(widthSegments), 1);
		heightSegments = MAX(ABS(heightSegments), 1);
		
		[self setDraggedObjectX:draggedObjectLocation.x y:draggedObjectLocation.y];
		id<FPGameObject> draggedObject = [self draggedObject];
		
		if ([draggedObject respondsToSelector:@selector(setWidthSegments:)])
			[draggedObject setWidthSegments:widthSegments];
		
		if ([draggedObject respondsToSelector:@selector(setHeightSegments:)])
			[draggedObject setHeightSegments:heightSegments];
	}
	
	[self setNeedsDisplay:YES];
}

- (void)mouseUp:(NSEvent *)theEvent
{
	if (drawingSelection)
	{
		NSMutableArray *gameObjects = [dataSource gameObjects];
		NSMutableIndexSet *selectedIndices = [dataSource selectedIndices];
		
		CGRect selectionRect = CGRectMakeFromPoints(beginSelection, endSelection);
		NSUInteger flags = [theEvent modifierFlags];		
		
		if ((flags & NSCommandKeyMask) == NSCommandKeyMask)
		{
			for (NSUInteger i = 0; i < [gameObjects count]; i++)
			{
				id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
				if (CGRectIntersectsRect(selectionRect, [gameObject rect]))
				{
					if ([selectedIndices containsIndex:i])
						[selectedIndices removeIndex:i];
					else
						[selectedIndices addIndex:i];
				}
			}
		}
		else
		{
			if ((flags & NSShiftKeyMask) != NSShiftKeyMask)
				[selectedIndices removeAllIndexes];
		
			for (NSUInteger i = 0; i < [gameObjects count]; i++)
			{
				id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
				if (CGRectIntersectsRect(selectionRect, [gameObject rect]))
				{
					[selectedIndices addIndex:i];
				}
			}
		}
		
		drawingSelection = NO;
	}
	else 
	{
		CGPoint move = endMovePoint;
		move.x -= beginMovePoint.x;
		move.y -= beginMovePoint.y;
		if (currentHandle == FPDragHandleCenter)
			[dataSource endMoveWithPoint:move];
		else if (currentHandle != FPDragHandleNone)
			[dataSource endResizeWithPoint:move];		
	}	
	
	[dataSource setActiveFactory:nil];
	[self setNeedsDisplay:YES];
}

#pragma mark Drawing

- (void)drawGrid
{
	CGRect rect = [self boundsCG];
	rect.origin.x = -rect.size.width;
	rect.origin.y = -rect.size.height;
	
	rect.origin.x = (int)(rect.origin.x) / 32;
	rect.origin.y = (int)(rect.origin.y) / 32;
	
	rect.origin.x *= 32.0f;
	rect.origin.y *= 32.0f;
	
	glDisable(GL_TEXTURE_2D);
	glBegin(GL_LINES);
	glColor4f(1, 1, 1, 0.2f);
	
	for (int y = rect.origin.y; y < rect.size.height; y += 32)
	{
		glVertex2i(rect.origin.x, y);	
		glVertex2i(rect.size.width, y);
	}
	
	for (int x = rect.origin.x; x < rect.size.width; x += 32)
	{
		glVertex2i(x, rect.origin.y);	
		glVertex2i(x, rect.size.height);
	}
	
	glEnd();
}

- (void)drawHandlesOnGameObject:(id<FPGameObject>)gameObject
{
	CGRect rect = [gameObject rect];
	
	glDisable(GL_TEXTURE_2D);
	glColor4f(1, 1, 1, 0.8f);
	glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
	glRectf(rect.origin.x, rect.origin.y, rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);
	glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
	
	glPointSize(6.0f);
	
	id<FPGameObject> draggedObject = [self draggedObject];
	
	BOOL widthHandles = NO;
	BOOL heightHandles = NO;
	
	if (draggedObject)
	{
		widthHandles = [draggedObject respondsToSelector:@selector(setWidthSegments:)];
		heightHandles = [draggedObject respondsToSelector:@selector(setHeightSegments:)];
	}
	
	glBegin(GL_POINTS);
	for (FPDragHandle handle = FPDragHandleTopLeft; handle < FPDragHandleCenter; handle++)
	{
		if (!widthHandles && [self isWidthHandle:handle])
			continue;
		
		if (!heightHandles && [self isHeightHandle:handle])
			continue;		
		
		if (handle == currentHandle)
			glColor4f(1, 0, 0, 0.8f);
		else
			glColor4f(1, 1, 0.5f, 0.8f);
		
		CGPoint handlePoint = [self pointFromHandle:handle aroundRect:rect];
		glVertex2f(handlePoint.x, handlePoint.y);
	}
	glEnd();
}

- (void)drawRect:(NSRect)dirtyRect 
{
	[self reshapeFlippedOrtho2D];
	
	glClear(GL_COLOR_BUFFER_BIT);
	
	[self drawGrid];
	
	glColor4f(1, 1, 1, 1);
	
	NSMutableArray *gameObjects = [dataSource gameObjects];
	NSMutableIndexSet *selectedIndices = [dataSource selectedIndices];
	
	glEnable(GL_BLEND);
	
	for (NSUInteger i = 0; i < [gameObjects count]; i++)
	{
		id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:i];
		glColor4f(1, 1, 1, 1);
		[gameObject draw];
		if ([selectedIndices containsIndex:i])
			[self drawHandlesOnGameObject:gameObject];
	}	
	
	id<FPGameObject> draggedObject = [self draggedObject];
	
	if (draggedObject)
		[self drawHandlesOnGameObject:draggedObject];
	
	if (drawingSelection)
	{
		glDisable(GL_TEXTURE_2D);
		glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
		glColor4f(1, 1, 1, 0.2f);
		glRectf(beginSelection.x, beginSelection.y, endSelection.x, endSelection.y);
		glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
		glColor4f(1, 1, 1, 0.9f);
		glRectf(beginSelection.x, beginSelection.y, endSelection.x, endSelection.y);
		glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
	}
	
	[[self openGLContext] flushBuffer];
}

#pragma mark Helpers

- (id<FPGameObject>)draggedObject
{
	NSMutableArray *gameObjects = [dataSource gameObjects];
	NSMutableIndexSet *selectedIndices = [dataSource selectedIndices];
	if ([selectedIndices count] == 1)
		return (id<FPGameObject>)[gameObjects objectAtIndex:[selectedIndices firstIndex]];
	return nil;
}

- (BOOL)respondsGameObject:(id<FPGameObject>)gameObject toDragHandle:(FPDragHandle)dragHandle
{
	BOOL respondsToWidth = [gameObject respondsToSelector:@selector(setWidthSegments:)];
	BOOL respondsToHeight = [gameObject respondsToSelector:@selector(setHeightSegments:)];
	
	if (!respondsToWidth && [self isWidthHandle:dragHandle])
		return NO;
	
	if (!respondsToHeight && [self isHeightHandle:dragHandle])
		return NO;
	
	return YES;
}

- (void)setDraggedObjectX:(float)x y:(float)y
{
	id<FPGameObject> draggedObject = [self draggedObject];
	[draggedObject moveWithX:x - draggedObject.x 
						   y:y - draggedObject.y];
}

- (void)resizeDraggedObjectLeft:(float)x
{
	id<FPGameObject> draggedObject = [self draggedObject];
	int widthSegments = (x - endMovePoint.x + 16.0f) / 32.0f;
	
	if (draggedObject.widthSegments - widthSegments < 1)
		widthSegments = 0;
	
	[draggedObject moveWithX:widthSegments * 32.0f y:0.0f];
	endMovePoint.x += widthSegments * 32.0f;
	draggedObject.widthSegments -= widthSegments;
}

- (void)resizeDraggedObjectRight:(float)x
{
	id<FPGameObject> draggedObject = [self draggedObject];
	int widthSegments = (x - endMovePoint.x + 16.0f) / 32.0f;
	
	if (draggedObject.widthSegments + widthSegments < 1)
		widthSegments = 0;
	
	endMovePoint.x += widthSegments * 32.0f;
	draggedObject.widthSegments += widthSegments;
}

- (void)resizeDraggedObjectTop:(float)y
{
	id<FPGameObject> draggedObject = [self draggedObject];							
	int heightSegments = (y - endMovePoint.y + 16.0f) / 32.0f;
	
	if (draggedObject.heightSegments - heightSegments < 1)
		heightSegments = 0;
	
	[draggedObject moveWithX:0.0f y:heightSegments * 32.0f];
	endMovePoint.y += heightSegments * 32.0f;
	draggedObject.heightSegments -= heightSegments;
}

- (void)resizeDraggedObjectBottom:(float)y
{
	id<FPGameObject> draggedObject = [self draggedObject];
	int heightSegments = (y - endMovePoint.y + 16.0f) / 32.0f;
	
	if (draggedObject.heightSegments + heightSegments < 1)
		heightSegments = 0;
	
	endMovePoint.y += heightSegments * 32.0f;
	draggedObject.heightSegments += heightSegments;
}

- (void)moveDraggedObjectWithX:(float)x y:(float)y
{
	id<FPGameObject> draggedObject = [self draggedObject];
	int widthSegments = (x - endMovePoint.x + 16.0f) / 32.0f;
	int heightSegments = (y - endMovePoint.y + 16.0f) / 32.0f;
	
	[draggedObject moveWithX:widthSegments * 32.0f y:heightSegments * 32.0f];
	endMovePoint.x += widthSegments * 32.0f;
	endMovePoint.y += heightSegments * 32.0f;
}

- (void)moveSelectedObjectsWithX:(float)x y:(float)y
{
	int widthSegments = (x - endMovePoint.x + 16.0f) / 32.0f;
	int heightSegments = (y - endMovePoint.y + 16.0f) / 32.0f;
	
	NSMutableArray *gameObjects = [dataSource gameObjects];
	NSMutableIndexSet *selectedIndices = [dataSource selectedIndices];
	[selectedIndices enumerateIndexesUsingBlock:^(NSUInteger idx, BOOL *stop) 
	{
		id<FPGameObject> gameObject = (id<FPGameObject>)[gameObjects objectAtIndex:idx];
		[gameObject moveWithX:widthSegments * 32.0f y:heightSegments * 32.0f];
	}];
	
	endMovePoint.x += widthSegments * 32.0f;
	endMovePoint.y += heightSegments * 32.0f;
}

- (BOOL)isWidthHandle:(FPDragHandle)handle
{
	switch (handle)
	{
		case FPDragHandleTopLeft:
		case FPDragHandleBottomLeft:
		case FPDragHandleTopRight:
		case FPDragHandleBottomRight:
		case FPDragHandleMiddleLeft:
		case FPDragHandleMiddleRight:
			return YES;
		case FPDragHandleMiddleTop:
		case FPDragHandleMiddleBottom:	
		case FPDragHandleCenter:
		case FPDragHandleNone:
		default:
			return NO;
	}
}

- (BOOL)isHeightHandle:(FPDragHandle)handle
{
	switch (handle)
	{
		case FPDragHandleTopLeft:
		case FPDragHandleBottomLeft:
		case FPDragHandleTopRight:
		case FPDragHandleBottomRight:
		case FPDragHandleMiddleTop:
		case FPDragHandleMiddleBottom:	
			return YES;
		case FPDragHandleMiddleLeft:
		case FPDragHandleMiddleRight:
		case FPDragHandleCenter:
		case FPDragHandleNone:
		default:
			return NO;
	}
}

- (CGPoint)pointFromHandle:(FPDragHandle)handle aroundRect:(CGRect)rect
{
	switch (handle)
	{
		case FPDragHandleTopLeft:
			return CGPointMake(rect.origin.x, rect.origin.y);
		case FPDragHandleBottomLeft:
			return CGPointMake(rect.origin.x, rect.origin.y + rect.size.height);
		case FPDragHandleTopRight:
			return CGPointMake(rect.origin.x + rect.size.width, rect.origin.y);
		case FPDragHandleBottomRight:
			return CGPointMake(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);
			
		case FPDragHandleMiddleLeft:
			return CGPointMake(rect.origin.x, rect.origin.y + rect.size.height / 2.0f);
		case FPDragHandleMiddleTop:
			return CGPointMake(rect.origin.x + rect.size.width / 2.0f, rect.origin.y);
		case FPDragHandleMiddleRight:
			return CGPointMake(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height / 2.0f);
		case FPDragHandleMiddleBottom:
			return CGPointMake(rect.origin.x + rect.size.width / 2.0f, rect.origin.y + rect.size.height);
		case FPDragHandleCenter:
			return CGPointMake(rect.origin.x + rect.size.width / 2.0f, rect.origin.y + rect.size.height / 2.0f);
			
		default:
			return CGPointZero;
	}
}

@end
